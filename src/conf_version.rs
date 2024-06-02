use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use serde::{Deserialize, Serialize};

/// `[term, version, index]` triple of a configration setting
/// TLA+ {D240508N-_ConfVersion}
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Decode,
    Encode
)]
pub struct ConfVersion {
    pub term: u64,
    pub version: u64,
    /// the commit index of the log, when re-config
    pub index: u64,
}

#[derive(
    Eq,
    PartialEq,
    Hash,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    Decode,
    Encode
)]
pub struct ConfVersionPair {
    pub conf_committed: ConfVersion,
    pub conf_new: ConfVersion,
}

impl MsgTrait for ConfVersion {}


impl MsgTrait for ConfVersionPair {}

impl Default for ConfVersion {
    fn default() -> Self {
        Self {
            term: 0,
            version: 0,
            index: 0,
        }
    }
}

impl Hash for ConfVersion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.term.hash(state);
        self.version.hash(state);
    }
}

impl PartialOrd for ConfVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConfVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.term.cmp(&other.term);
        if ord.is_eq() {
            self.version.cmp(&other.version)
        } else {
            ord
        }
    }
}

impl PartialEq for ConfVersion {
    fn eq(&self, other: &Self) -> bool {
        self.term.eq(&other.term) &&
            self.version.eq(&other.version)
    }
}

impl Eq for ConfVersion {}