use arbitrary::Arbitrary;
use scupt_util::message::MsgTrait;
use scupt_util::node_id::NID;

use crate::conf_value::ConfValue;
use crate::conf_version::ConfVersion;
use crate::log_entry::LogEntry;
use crate::snapshot::SnapshotRange;

#[derive(Clone, Debug)]
pub enum NonVolatileWrite<T: MsgTrait + 'static> {
    OpUpConfCommitted { value: ConfValue, version: ConfVersion },
    OpUpConfNew { value: ConfValue, version: ConfVersion },
    OpUpCommitIndex(u64),
    /// term value, optional voted for NID
    OpUpTermVotedFor { term: u64, voted_for: Option<NID> },
    /// previous log index, an array of log entries
    OpWriteLog { prev_index: u64, entries: Vec<LogEntry<T>>, opt: WriteEntriesOpt },
    OpCompactLog(Vec<LogEntry<T>>),
    OpApplySnapshot(SnapshotRange<T>),
}

/// state machine store write log entries option parameter
#[derive(Clone, Debug, Arbitrary)]
pub struct WriteEntriesOpt {
    /// truncate left side of the log writing position
    pub truncate_left: bool,

    /// truncate right side of the log writing position
    pub truncate_right: bool,
}

/// state machine store write snapshot log entries option parameter
/// #[derive(Clone, Debug, Arbitrary)]
pub struct WriteSnapshotOpt {
    /// truncate left side of the snapshot index position
    /// reserve
    #[allow(dead_code)]
    pub truncate_left: bool,
    #[allow(dead_code)]
    /// truncate right side of the snapshot index position
    pub truncate_right: bool,
}

impl Default for WriteEntriesOpt {
    fn default() -> Self {
        Self {
            truncate_right: true,
            truncate_left: false,
        }
    }
}

impl Default for WriteSnapshotOpt {
    fn default() -> Self {
        Self {
            truncate_left: true,
            truncate_right: false,
        }
    }
}
