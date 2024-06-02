use std::ops::Bound;

use async_trait::async_trait;
use scupt_util::message::MsgTrait;
use scupt_util::node_id::NID;
use scupt_util::res::Res;

use crate::conf_value::ConfValue;
use crate::conf_version::ConfVersion;
use crate::non_volatile_write::NonVolatileWrite;
use crate::raft_message::LogEntry;
use crate::term_index::TermIndex;

#[async_trait]
pub trait StoreAsync<T: MsgTrait + 'static> {
    /// cluster configure,
    /// return `[configration committed, configration new]`
    async fn conf(&self) -> Res<((ConfValue, ConfVersion), (ConfValue, ConfVersion))>;

    /// get current (term, voted for node id) pair
    async fn term_and_voted_for(&self) -> Res<(u64, Option<NID>)>;

    /// the maximum log index number of un-compacted log entries
    async fn max_log_index(&self) -> Res<u64>;

    /// the minimum log index number of un-compacted log entries
    async fn min_log_index(&self) -> Res<u64>;

    /// retrieve the log entries in range [start, end)
    async fn read_log_entries(&self, start: Bound<u64>, end: Bound<u64>) -> Res<Vec<LogEntry<T>>>;

    async fn snapshot_index_term(&self) -> Res<TermIndex>;


    /// `index`, index to start read
    /// `limit`, read values
    async fn read_snapshot_value(
        &self,
        index: u64,
        limit: Option<u64>,
    ) -> Res<Vec<LogEntry<T>>>;


    /// write operations
    async fn write(&self,
                   operations: Vec<NonVolatileWrite<T>>,
    ) -> Res<()>;
}
