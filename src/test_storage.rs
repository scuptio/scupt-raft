pub mod tests {
    use std::collections::HashSet;
    use std::sync::Arc;

    use arbitrary::{Arbitrary, Unstructured};
    use scupt_util::message::MsgTrait;
    use scupt_util::mt_set::MTSet;
    use scupt_util::node_id::NID;
    use scupt_util::res::Res;
    use tokio::runtime::Builder;

    use crate::conf_value::ConfValue;
    use crate::conf_version::ConfVersion;
    use crate::log_entry::LogEntry;
    use crate::node_info::NodeInfo;
    use crate::non_volatile_write::{NonVolatileWrite, WriteEntriesOpt};
    use crate::snapshot::SnapshotRange;
    use crate::storage::Storage;
    use crate::test_store_simple::tests::SMStoreSimple;

    const ARBITRARY_MAX_NODES: u64 = 20;
    const ARBITRARY_MAX_ENTRIES: u64 = 10;

    struct _Arbitrary {
        nid: Vec<NID>,
    }


    fn arbitrary_index(u: &mut Unstructured) -> arbitrary::Result<u64> {
        let n = u64::arbitrary(u)?;
        Ok(n % (ARBITRARY_MAX_NODES * 2))
    }


    fn arbitrary_term(u: &mut Unstructured) -> arbitrary::Result<u64> {
        let n = u64::arbitrary(u)?;
        Ok(n % (ARBITRARY_MAX_NODES * 2))
    }


    fn arbitrary_version(u: &mut Unstructured) -> arbitrary::Result<u64> {
        let n = u64::arbitrary(u)?;
        Ok(n % (ARBITRARY_MAX_NODES * 2))
    }


    fn arbitrary_nid(u: &mut Unstructured) -> arbitrary::Result<NID> {
        let nid = NID::arbitrary(u)? % ARBITRARY_MAX_NODES + 1;
        Ok(nid)
    }


    fn arbitrary_conf_version(u: &mut Unstructured) -> arbitrary::Result<ConfVersion> {
        let cv = ConfVersion {
            term: arbitrary_term(u)?,
            version: arbitrary_version(u)?,
            index: arbitrary_index(u)?,
        };
        Ok(cv)
    }


    fn arbitrary_log_entries<T: for<'a> Arbitrary<'a> + MsgTrait + 'static>(
        prev_index: u64,
        term: u64,
        u: &mut Unstructured,
    ) -> arbitrary::Result<Vec<LogEntry<T>>> {
        let n = u32::arbitrary(u)? % (ARBITRARY_MAX_ENTRIES as u32);
        let mut vec = vec![];
        for i in 0..(n as u64) {
            let e = LogEntry {
                term,
                index: prev_index + i + 1,
                value: T::arbitrary(u)?,
            };
            vec.push(e);
        }
        Ok(vec)
    }


    fn arbitrary_term_voted_for(u: &mut Unstructured) -> arbitrary::Result<(u64, Option<NID>)> {
        let _n = u64::arbitrary(u)?;
        let nid = arbitrary_nid(u)?;
        let opt_nid = if u8::arbitrary(u)? % 2 == 0 {
            Some(nid)
        } else {
            None
        };
        let term = arbitrary_term(u)?;
        Ok((term, opt_nid))
    }


    impl<'a, T: for<'b> Arbitrary<'b> + MsgTrait + 'static> Arbitrary<'a> for NonVolatileWrite<T> {

        fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
            let n = u8::arbitrary(u)?;
            let write_op = match n % 7 {
                0 => {
                    Self::OpUpCommitIndex(arbitrary_index(u)?)
                }
                1 => {
                    let (term, voted_for) = arbitrary_term_voted_for(u)?;
                    Self::OpUpTermVotedFor { term, voted_for }
                }
                2 => {
                    let prev_index = arbitrary_index(u)?;
                    let term = arbitrary_term(u)?;
                    let entries = arbitrary_log_entries(prev_index, term, u)?;
                    Self::OpWriteLog {
                        prev_index,
                        entries,
                        opt: WriteEntriesOpt::arbitrary(u)?,
                    }
                }
                3 => {
                    let prev_index = arbitrary_index(u)?;
                    let term = arbitrary_term(u)?;
                    let entries: Vec<LogEntry<T>> = arbitrary_log_entries(prev_index, term, u)?;
                    let (begin, end) = if entries.is_empty() {
                        (0, 0)
                    } else {
                        (entries.first().unwrap().index, entries.last().unwrap().index + 1)
                    };

                    let set: HashSet<LogEntry<_>> = entries.into_iter().collect();
                    let snapshot = SnapshotRange {
                        end_index: end,
                        begin_index: begin,
                        entries: MTSet::new(set),
                    };
                    Self::OpApplySnapshot(snapshot)
                }
                4 => {
                    let prev_index = arbitrary_index(u)?;
                    let term = arbitrary_term(u)?;
                    let entries = arbitrary_log_entries(prev_index, term, u)?;
                    Self::OpCompactLog(entries)
                }
                5 => {
                    let version = arbitrary_conf_version(u)?;
                    let value = ConfValue::default();
                    Self::OpUpConfNew { value, version }
                }
                _ => {
                    let version = arbitrary_conf_version(u)?;
                    let value = ConfValue::default();
                    Self::OpUpConfCommitted { value, version }
                }
            };
            Ok(write_op)
        }
    }

    #[derive(Clone, Arbitrary, Debug)]
    pub struct StorageOperation {
        vec: Vec<NonVolatileWrite<u64>>,
    }


    async fn _test_write_log(ops: Vec<NonVolatileWrite<u64>>, storage: Storage<u64>) -> Res<()> {
        storage.write(ops).await
    }


    pub fn _test_storage(ops: StorageOperation) {
        let conf = ConfValue {
            cluster_name: "test_storage".to_string(),
            storage_path: "/tmp/test_storage".to_string(),
            node_id: 1,
            bind_address: "0.0.0.0".to_string(),
            bind_port: 10888,
            timeout_max_tick: 10,
            millisecond_tick: 20,
            max_compact_entries: 10,
            send_value_to_leader: false,
            node_peer: vec![
                NodeInfo {
                    node_id: 2,
                    can_vote: true,
                },
                NodeInfo {
                    node_id: 3,
                    can_vote: true,
                },
            ],
        };
        let storage = Storage::Async(Arc::new(SMStoreSimple::<u64>::create(conf).unwrap()));
        let runtime = Builder::new_current_thread().build().unwrap();
        runtime.block_on(async move {
            let _ = _test_write_log(ops.vec, storage).await;
        })
    }
}
