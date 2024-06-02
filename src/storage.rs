use std::collections::Bound;
use std::sync::Arc;

use scupt_util::message::MsgTrait;
use scupt_util::node_id::NID;
use scupt_util::res::Res;

use crate::conf_value::ConfValue;
use crate::conf_version::ConfVersion;
use crate::non_volatile_write::NonVolatileWrite;
use crate::raft_message::LogEntry;
use crate::store_async::StoreAsync;
use crate::store_sync::StoreSync;
use crate::term_index::TermIndex;

#[derive(Clone)]
pub enum Storage<T> {
    Async(Arc<dyn StoreAsync<T>>),
    Sync(Arc<dyn StoreSync<T>>),
}

impl<T: MsgTrait + 'static> Storage<T> {
    /// cluster configure,
    /// return `[configration committed, configration new]`
    pub async fn conf(&self) -> Res<((ConfValue, ConfVersion), (ConfValue, ConfVersion))> {
        match self {
            Storage::Async(s) => { s.conf().await }
            Storage::Sync(s) => { s.conf() }
        }
    }

    /// get current (term, voted for node id) pair
    pub async fn term_and_voted_for(&self) -> Res<(u64, Option<NID>)> {
        match self {
            Storage::Async(s) => { s.term_and_voted_for().await }
            Storage::Sync(s) => { s.term_and_voted_for() }
        }
    }

    /// the maximum log index number of un-compacted log entries
    pub async fn max_log_index(&self) -> Res<u64> {
        match self {
            Storage::Async(s) => { s.max_log_index().await }
            Storage::Sync(s) => { s.max_log_index() }
        }
    }

    /// the minimum log index number of un-compacted log entries
    pub async fn min_log_index(&self) -> Res<u64> {
        match self {
            Storage::Async(s) => { s.min_log_index().await }
            Storage::Sync(s) => { s.min_log_index() }
        }
    }

    /// retrieve the log entries in range [start, end)
    pub async fn read_log_entries(&self, start: Bound<u64>, end: Bound<u64>) -> Res<Vec<LogEntry<T>>> {
        match self {
            Storage::Async(s) => { s.read_log_entries(start, end).await }
            Storage::Sync(s) => { s.read_log_entries(start, end) }
        }
    }

    pub async fn snapshot_index_term(&self) -> Res<TermIndex> {
        match self {
            Storage::Async(s) => { s.snapshot_index_term().await }
            Storage::Sync(s) => { s.snapshot_index_term() }
        }
    }


    /// `index`, index to start read
    /// `limit`, read values
    pub async fn read_snapshot_value(
        &self,
        index: u64,
        limit: Option<u64>,
    ) -> Res<Vec<LogEntry<T>>> {
        match self {
            Storage::Async(s) => { s.read_snapshot_value(index, limit).await }
            Storage::Sync(s) => { s.read_snapshot_value(index, limit) }
        }
    }

    pub async fn write(&self,
                       operations: Vec<NonVolatileWrite<T>>,
    ) -> Res<()> {
        match self {
            Storage::Async(s) => { s.write(operations).await }
            Storage::Sync(s) => { s.write(operations) }
        }
    }
}