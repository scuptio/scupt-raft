pub mod tests {
    use std::collections::{Bound, HashMap};
    use std::marker::PhantomData;
    use std::sync::Arc;

    use async_trait::async_trait;
    use rusqlite::{Connection, params, Transaction};
    use scupt_util::error_type::ET;
    use scupt_util::message::MsgTrait;
    use scupt_util::node_id::NID;
    use scupt_util::res::Res;
    use scupt_util::res_of::{res_option, res_sqlite};
    use serde::{Deserialize, Serialize};
    use serde::de::DeserializeOwned;
    use tracing::error;

    use crate::conf_value::ConfValue;
    use crate::conf_version::ConfVersion;
    use crate::non_volatile_write::{NonVolatileWrite, WriteEntriesOpt, WriteSnapshotOpt};
    use crate::raft_message::LogEntry;
    use crate::store_async::StoreAsync;
    use crate::term_index::TermIndex;
    use crate::test_check_invariants::tests::InvariantChecker;

    struct StoreInner<T: MsgTrait + 'static> {
        p: PhantomData<T>,
        path: String,
        cluster_name: String,
    }

    pub struct SMStoreSimple<T: MsgTrait + 'static> {
        enable_check_invariants: bool,
        inner: Arc<StoreInner<T>>,
    }


    impl<T: MsgTrait + 'static> SMStoreSimple<T> {
        #[allow(dead_code)]
        pub fn open(storage_path: String, cluster_name: String) -> Res<Self> {
            let inner = StoreInner::<T>::new(
                storage_path.clone(),
                cluster_name.clone())?;
            let s = Self {
                enable_check_invariants: false,
                inner: Arc::new(inner),
            };
            Ok(s)
        }

        pub fn create(conf: ConfValue) -> Res<Self> {
            let inner = StoreInner::<T>::new(
                conf.storage_path.clone(),
                conf.cluster_name.clone())?;
            let version = ConfVersion::default();
            inner.write_conf_committed((conf.clone(), version.clone()))?;
            inner.write_conf_new((conf.clone(), version.clone()))?;
            let s = Self {
                enable_check_invariants: false,
                inner: Arc::new(inner),
            };
            Ok(s)
        }

        async fn changed_term_voted_check(&self) -> Res<()> {
            if !self.enable_check_invariants {
                return Ok(());
            }
            let (term, voted_for) = self.inner.get_term_voted()?;
            InvariantChecker::set_and_check_invariants::<T>(
                self.inner.cluster_name.clone(),
                self.inner.path.clone(),
                None,
                None,
                Some((term, voted_for)),
                None,
                true,
            );
            Ok(())
        }

        async fn changed_commit_index_check(&self) -> Res<()> {
            if !self.enable_check_invariants {
                return Ok(());
            }
            let index = self.inner.get_commit_index()?;
            InvariantChecker::update_commit_index(
                self.inner.cluster_name.clone(),
                self.inner.path.clone(),
                index);

            Ok(())
        }


        fn read_snapshot_value(&self) -> Res<Vec<LogEntry<T>>> {
            let vec: Vec<LogEntry<T>> = self.inner.read_snapshot(Bound::Unbounded, Bound::Unbounded)?;
            Ok(vec)
        }



    }

    #[async_trait]
    impl<T: MsgTrait + 'static> StoreAsync<T> for SMStoreSimple<T> {
        async fn conf(&self) -> Res<((ConfValue, ConfVersion), (ConfValue, ConfVersion))> {
            Ok((self.inner.read_conf_committed()?, self.inner.read_conf_new()?))
        }

        async fn term_and_voted_for(&self) -> Res<(u64, Option<NID>)> {
            self.inner.get_term_voted()
        }

        async fn max_log_index(&self) -> Res<u64> {
            self.inner.get_max_log_index()
        }

        async fn min_log_index(&self) -> Res<u64> {
            self.inner.get_min_log_index()
        }

        async fn read_log_entries(&self, start: Bound<u64>, end: Bound<u64>) -> Res<Vec<LogEntry<T>>> {
            self.inner.read_log_entries(start, end)
        }

        async fn snapshot_index_term(&self) -> Res<TermIndex> {
            self.inner.read_index_term()
        }

        async fn read_snapshot_value(&self, _index: u64, _limit: Option<u64>) -> Res<Vec<LogEntry<T>>> {
            self.read_snapshot_value()
        }

        async fn write(&self, operations: Vec<NonVolatileWrite<T>>) -> Res<()> {
            for op in operations {
                match op {
                    NonVolatileWrite::OpUpConfCommitted { value, version } => {
                        self.inner.write_conf_committed((value, version))?;
                    }
                    NonVolatileWrite::OpUpConfNew { value, version } => {
                        self.inner.write_conf_new((value, version))?;
                    }
                    NonVolatileWrite::OpUpCommitIndex(index) => {
                        self.inner.set_commit_index(index)?;
                        self.changed_commit_index_check().await?;
                    }
                    NonVolatileWrite::OpUpTermVotedFor { term, voted_for } => {
                        self.inner.set_term_voted(term, voted_for)?;
                        self.changed_term_voted_check().await?;
                    }
                    NonVolatileWrite::OpWriteLog { prev_index, entries, opt } => {
                        self.inner.write_log_entries(prev_index, entries, opt)?;
                    }
                    NonVolatileWrite::OpCompactLog(entries) => {
                        self.inner.compact_log(entries).await?;
                    }
                    NonVolatileWrite::OpApplySnapshot(snapshot) => {
                        let value = snapshot.entries.zzz_array;
                        self.inner.apply_snapshot("".to_string(), value, WriteSnapshotOpt::default()).await?;
                    }
                }
            }
            Ok(())
        }
    }


    impl<T: MsgTrait + 'static> StoreInner<T> {
        fn new(path: String, cluster_name: String) -> Res<Self> {
            let s = Self {
                p: Default::default(),
                path,
                cluster_name,
            };
            s.open()?;
            Ok(s)
        }

        async fn compact_log(&self, entries: Vec<LogEntry<T>>) -> Res<()> {
            let snapshot_values = self.read_snapshot(Bound::Unbounded, Bound::Unbounded)?;
            let mut values = HashMap::new();
            let max_index = if let Some(e) = entries.last() {
                e.index
            } else {
                return Ok(());
            };
            for v in snapshot_values {
                values.insert(v.value.clone(), v);
            }

            for e in entries {
                values.insert(e.value.clone(), e);
            }

            let mut vec = vec![];
            for (_k, v) in values {
                vec.push(v);
            }
            // * remove all log entries in log
            self.write_snapshot(vec, Some(max_index), None).await?;
            Ok(())
        }

        pub async fn write_snapshot(
            &self,
            values: Vec<LogEntry<T>>,
            truncate_left: Option<u64>,
            truncate_right: Option<u64>) -> Res<()> {
            let mut conn = self.connection()?;
            let trans = res_sqlite(conn.transaction())?;
            self.write_snapshot_gut(&trans, values,
                                    truncate_left, truncate_right)?;
            res_sqlite(trans.commit())?;
            Ok(())
        }

        fn write_conf_committed(&self, conf: (ConfValue, ConfVersion)) -> Res<()> {
            self.put_key("conf_committed".to_string(), conf)?;
            Ok(())
        }


        fn read_conf_committed(&self) -> Res<(ConfValue, ConfVersion)> {
            let conf = res_option(self.query_key("conf_committed".to_string())?)?;
            Ok(conf)
        }

        fn write_conf_new(&self, conf: (ConfValue, ConfVersion)) -> Res<()> {
            self.put_key("conf_new".to_string(), conf)?;
            Ok(())
        }

        ///
        fn read_conf_new(&self) -> Res<(ConfValue, ConfVersion)> {
            let conf:(ConfValue, ConfVersion) = res_option(self.query_key("conf_new".to_string())?)?;
            Ok(conf)
        }

        fn set_commit_index(&self, commit_index: u64) -> Res<()> {
            let mut conn = self.connection()?;
            let trans = res_sqlite(conn.transaction())?;
            self._set_commit_index(&trans, commit_index)?;
            trans.commit().unwrap();
            Ok(())
        }


        fn set_term_voted(&self, term: u64, voted_for: Option<NID>) -> Res<()> {
            let mut conn = self.connection()?;
            let trans = res_sqlite(conn.transaction())?;
            self._set_term_voted(&trans, term, voted_for)?;
            trans.commit().unwrap();
            Ok(())
        }

        fn _set_commit_index(&self, trans: &Transaction, commit_index: u64) -> Res<()> {
            self.trans_put_key(trans, "commit_index".to_string(), commit_index)?;
            Ok(())
        }

        /// update term and voted_for
        fn _set_term_voted(&self, trans: &Transaction, term: u64, voted_for: Option<NID>) -> Res<()> {
            let t = TermVotedFor {
                term,
                voted_for,
            };
            self.trans_put_key(trans, "term_voted_for".to_string(), t)?;
            Ok(())
        }

        fn get_commit_index(&self) -> Res<u64> {
            let opt: Option<u64> = self.query_key("commit_index".to_string())?;
            if let Some(i) = opt {
                Ok(i)
            } else {
                Ok(0)
            }
        }

        /// get current (term, voted_for)
        fn get_term_voted(&self) -> Res<(u64, Option<NID>)> {
            let opt: Option<TermVotedFor> = self.query_key("term_voted_for".to_string())?;
            if let Some(t) = opt {
                Ok((t.term, t.voted_for))
            } else {
                Ok((0, None))
            }
        }

        fn get_max_log_index(&self) -> Res<u64> {
            self.max_log_index()
        }


        fn get_min_log_index(&self) -> Res<u64> {
            self.min_log_index()
        }

        /// simple store does not implement iterator
        /// write_snapshot would be atomic in a single invoke
        async fn apply_snapshot(
            &self, _id: String,
            values: Vec<LogEntry<T>>,
            _write_opt: WriteSnapshotOpt,
        ) -> Res<Option<Vec<u8>>> {
            let truncate_left = None;
            let truncate_right = None;
            let mut conn = self.connection()?;
            let trans = res_sqlite(conn.transaction())?;
            self.write_snapshot_gut(&trans, values, truncate_left, truncate_right)?;
            trans.commit().unwrap();
            Ok(None)
        }

        /// the log would be truncated by index, > index entries would be left; and then the new add entries
        /// would be appended
        fn write_log_entries(&self, prev_index: u64, log_entries: Vec<LogEntry<T>>, opt: WriteEntriesOpt) -> Res<()> {
            let mut conn = self.connection().unwrap();
            let trans = conn.transaction().unwrap();
            self.write_log_gut(&trans, prev_index, log_entries, opt)?;
            trans.commit().unwrap();
            Ok(())
        }

        /// retrieve the log entries in range [start, end)
        fn read_log_entries(&self, start: Bound<u64>, end: Bound<u64>) -> Res<Vec<LogEntry<T>>> {
            self.read_log(start, end)
        }


        fn read_index_term(&self) -> Res<TermIndex> {
            // select the maximum index log entry
            let sql = "select entry_index, entry_term \
                from snapshot \
                order by entry_index desc limit 0, 1;".to_string();
            let (index, term) = self.query_rows_index_term(sql)?;
            Ok(TermIndex {
                index,
                term,
            })
        }

        fn query_key<M: DeserializeOwned + Serialize + 'static>(&self, key: String) -> Res<Option<M>> {
            let c = res_sqlite(Connection::open(self.path.clone()))?;
            let mut s = res_sqlite(c.prepare("select json_value from meta where json_key = ?;"))?;
            let mut cursor = s.query([key]).unwrap();

            if let Some(row) = res_sqlite(cursor.next())? {
                let r = row.get(0);
                let s: String = match r {
                    Ok(s) => { s }
                    Err(e) => {
                        error!("error get index 0, {:?}", e);
                        panic!("error");
                    }
                };
                let m: M = serde_json::from_str(s.as_str()).unwrap();
                Ok(Some(m))
            } else {
                Ok(None)
            }
        }

        fn connection(&self) -> Res<Connection> {
            let c = res_sqlite(Connection::open(self.path.clone()))?;
            Ok(c)
        }

        fn put_key<M: DeserializeOwned + Serialize + 'static>(&self, key: String, value: M) -> Res<()> {
            let mut conn = self.connection()?;
            let trans = res_sqlite(conn.transaction())?;
            self.trans_put_key(&trans, key, value)?;
            res_sqlite(trans.commit())?;
            Ok(())
        }


        fn trans_put_key<M: DeserializeOwned + Serialize + 'static>(&self, tran: &Transaction, key: String, value: M) -> Res<()> {
            let json = serde_json::to_string(&value).unwrap();
            let _ = tran.execute(r#"insert into meta(json_key, json_value) values(?1, ?2)
        on conflict(json_key) do update set json_value=?3;"#,
                                 [key.clone(), json.clone(), json.clone()]).unwrap();
            Ok(())
        }

        fn min_log_index(&self) -> Res<u64> {
            let sql = "select entry_index, entry_term \
                from log \
                order by entry_index asc limit 0, 1;".to_string();
            let (index, _) = self.query_rows_index_term(sql)?;
            return Ok(index);
        }


        fn max_log_index(&self) -> Res<u64> {
            let sql =
                "select entry_index, entry_term \
                from log \
                order by entry_index desc limit 0, 1;".to_string();
            let (index, _) = self.query_rows_index_term(sql)?;
            return Ok(index);
        }

        fn query_rows_index_term(&self, sql: String) -> Res<(u64, u64)> {
            let mut c = Connection::open(self.path.clone()).unwrap();
            let t = c.transaction().unwrap();
            let mut stmt = res_sqlite(t.prepare(sql.as_str()))?;
            let mut cursor = res_sqlite(stmt.query([]))?;
            let opt_row = res_sqlite(cursor.next())?;
            match opt_row {
                None => { Ok((0, 0)) }
                Some(row) => {
                    let index: u64 = row.get(0).unwrap();
                    let term: u64 = row.get(1).unwrap();
                    Ok((index, term))
                }
            }
        }
        pub fn write_snapshot_gut(
            &self,
            trans: &Transaction<'_>,
            values: Vec<LogEntry<T>>,
            truncate_left: Option<u64>,
            truncate_right: Option<u64>,
        ) -> Res<()> {
            if let Some(_i) = truncate_left {
                trans_truncate_left(&trans, _i)?;
            }
            if let Some(_i) = truncate_right {
                trans_truncate_right_open(&trans, _i)?;
            }

            self.write_entries(trans, None, values, "snapshot".to_string())?;

            Ok(())
        }

        fn write_entries(
            &self, trans: &Transaction,
            opt_prev_index: Option<u64>,
            log_entries: Vec<LogEntry<T>>,
            table_name: String,
        ) -> Res<()> {
            let mut _prev_index = match opt_prev_index {
                Some(i) => { i }
                None => { 0 }
            };

            for l in log_entries {
                if opt_prev_index.is_some() {
                    if _prev_index + 1 != l.index {
                        return Err(ET::FatalError("error log entry index".to_string()));
                    }
                    _prev_index += 1;
                }
                let json = serde_json::to_string(&l).unwrap();
                let _ = res_sqlite(trans.execute(
                    format!("insert into {} (
                        entry_index,
                        entry_term,
                        entry_payload) values(?1, ?2, ?3)
                on conflict(entry_index)
                    do update
                    set
                        entry_term=?4,
                        entry_payload=?5;
                        ", table_name).as_str(),
                    params![
                        l.index,
                        l.term,
                        json.clone(),
                        l.term,
                        json.clone()
                ],
                ))?;
            }
            Ok(())
        }

        fn write_log_gut(&self, trans: &Transaction, prev_index: u64, log_entries: Vec<LogEntry<T>>, opt: WriteEntriesOpt) -> Res<()> {
            if opt.truncate_left {
                trans_truncate_left(trans, prev_index)?;
            }
            if opt.truncate_right {
                trans_truncate_right_open(trans, prev_index + (log_entries.len() as u64))?;
            }

            self.write_entries(trans, Some(prev_index), log_entries, "log".to_string())
        }


        fn read_entries(&self, start: Bound<u64>, end: Bound<u64>, table_name: String) -> Res<Vec<LogEntry<T>>> {
            let mut c = Connection::open(self.path.clone()).unwrap();
            let start_pred = match start {
                Bound::Included(x) => { format!(" entry_index >= {} ", x) }
                Bound::Excluded(x) => { format!(" entry_index > {} ", x) }
                Bound::Unbounded => { " true ".to_string() }
            };

            let end_pred = match end {
                Bound::Included(x) => { format!(" entry_index <= {} ", x) }
                Bound::Excluded(x) => { format!(" entry_index < {} ", x) }
                Bound::Unbounded => { " true ".to_string() }
            };

            let mut vec = vec![];
            let t = c.transaction().unwrap();
            {
                let mut s = t.prepare(
                    format!("select entry_index, entry_term, entry_payload
                    from {} where {} and {}", table_name, start_pred, end_pred).as_str()).unwrap();
                let mut rows = s.query(params![]).unwrap();

                while let Ok(row) = rows.next() {
                    if let Some(r) = row {
                        let s = r.get::<_, String>(2).unwrap();
                        let e: LogEntry<T> = serde_json::from_str(s.as_str()).unwrap();
                        let entry = LogEntry {
                            term: r.get::<_, u64>(1).unwrap(),
                            index: r.get::<_, u64>(0).unwrap(),
                            value: e.value,
                        };
                        vec.push(entry);
                    } else {
                        break;
                    }
                }
            }
            t.commit().unwrap();
            Ok(vec)
        }

        fn read_snapshot(&self, start: Bound<u64>, end: Bound<u64>) -> Res<Vec<LogEntry<T>>> {
            self.read_entries(start, end, "snapshot".to_string())
        }

        fn read_log(&self, start: Bound<u64>, end: Bound<u64>) -> Res<Vec<LogEntry<T>>> {
            self.read_entries(start, end, "log".to_string())
        }

        fn open(&self) -> Res<()> {
            let mut c = Connection::open(self.path.clone()).unwrap();
            let t = c.transaction().unwrap();
            // table log and snapshot have save structure
            res_sqlite(t.execute(
                r#"create table if not exists log (
                    entry_index integer primary key,
                    entry_term integer not null,
                    entry_payload text not null
                ) strict;"#, []))?;
            res_sqlite(t.execute(
                r#"create table if not exists snapshot (
                    entry_index integer primary key,
                    entry_term integer not null,
                    entry_payload text not null
                ); strict"#, []))?;
            res_sqlite(t.execute(
                r#"create table if not exists meta (
                    json_key text primary key,
                    json_value text not null
                ); strict"#, []))?;
            res_sqlite(t.commit())?;
            Ok(())
        }
    }


    fn trans_truncate_left(trans: &Transaction, index: u64) -> Res<()> {
        res_sqlite(trans.execute(
            r#"delete from log
                where entry_index <= ?1;"#,
            params![index],
        ))?;
        Ok(())
    }

    fn trans_truncate_right_open(trans: &Transaction, index: u64) -> Res<()> {
        res_sqlite(trans.execute(
            r#"delete from log
                where entry_index > ?1;"#,
            params![index],
        ))?;
        Ok(())
    }

    #[derive(
        Serialize,
        Deserialize,
    )]
    pub struct TermVotedFor {
        pub term: u64,
        pub voted_for: Option<NID>,
    }
}

