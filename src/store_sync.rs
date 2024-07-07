use std::ops::Bound;

use scupt_util::message::MsgTrait;
use scupt_util::node_id::NID;
use scupt_util::res::Res;

use crate::conf_value::ConfValue;
use crate::conf_version::ConfVersion;
use crate::log_entry::LogEntry;
use crate::non_volatile_write::NonVolatileWrite;
use crate::term_index::TermIndex;

pub trait StoreSync<T: MsgTrait + 'static> {
    /// cluster configure
    fn conf(&self) -> Res<((ConfValue, ConfVersion), (ConfValue, ConfVersion))>;

    /// get current (term, voted for node id) pair
    fn term_and_voted_for(&self) -> Res<(u64, Option<NID>)>;

    /// the maximum log index number of un-compacted log entries
    fn max_log_index(&self) -> Res<u64>;

    /// the minimum log index number of un-compacted log entries
    fn min_log_index(&self) -> Res<u64>;

    /// retrieve the log entries in range [start, end)
    fn read_log_entries(&self, start: Bound<u64>, end: Bound<u64>) -> Res<Vec<LogEntry<T>>>;


    fn snapshot_index_term(&self) -> Res<TermIndex>;


    /// `index`, index to start read
    /// `limit`, read values
    fn read_snapshot_value(
        &self,
        index: u64,
        limit: Option<u64>,
    ) -> Res<Vec<LogEntry<T>>>;

    fn write(&self,
             operations: Vec<NonVolatileWrite<T>>,
    ) -> Res<()>;
}