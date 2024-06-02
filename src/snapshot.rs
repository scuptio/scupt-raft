use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use scupt_util::mt_set::MTSet;
use serde::{Deserialize, Serialize};

use crate::raft_message::LogEntry;
use crate::term_index::TermIndex;

/// SnapshotV use by DTM
#[derive(
    Clone,
    Hash,
    PartialEq,
    Eq,
    Debug,
    Serialize,
    Deserialize,
    Decode,
    Encode,
)]
pub struct SnapshotRange<V: MsgTrait + 'static> {
    pub begin_index: u64,
    pub end_index: u64,
    #[serde(bound = "V: MsgTrait")]
    pub entries: MTSet<LogEntry<V>>,
}


/// SnapshotV use by DTM
#[derive(
    Default,
    Clone,
    Hash,
    PartialEq,
    Eq,
    Debug,
    Serialize,
    Deserialize,
    Decode,
    Encode,
)]
pub struct Snapshot<V: MsgTrait + 'static> {
    pub index: u64,
    pub term: u64,
    #[serde(bound = "V: MsgTrait")]
    pub entries: MTSet<LogEntry<V>>,
}


impl<V: MsgTrait + 'static> MsgTrait for SnapshotRange<V> {}

impl<V: MsgTrait + 'static> Default for SnapshotRange<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: MsgTrait + 'static> SnapshotRange<V> {
    pub fn new() -> Self {
        Self {
            begin_index: 0,
            end_index: 0,
            entries: MTSet::default(),
        }
    }

    pub fn to_snapshot_index_term(&self) -> TermIndex {
        let mut entries = self.entries.zzz_array.clone();
        entries.sort_by(|x, y| {
            x.index.cmp(&y.index)
        });
        if let Some(e) = entries.last() {
            if e.index == self.end_index {
                TermIndex {
                    term: e.term,
                    index: self.end_index,
                }
            } else {
                panic!("last index must the max index")
            }
        } else {
            TermIndex {
                term: 0,
                index: 0,
            }
        }
    }

    pub fn to_value(&self) -> Vec<LogEntry<V>> {
        let vec: Vec<LogEntry<V>> = self.entries.to_set().into_iter().collect();
        vec
    }

    pub fn map<V2, F>(&self, f: F) -> SnapshotRange<V2>
        where V2: MsgTrait + 'static,
              F: Fn(&V) -> V2
    {
        let set1 = self.entries.to_set();
        let set2 = set1.iter().map(|v| {
            LogEntry {
                index: v.index,
                term: v.term,
                value: f(&v.value),
            }
        }).collect();

        SnapshotRange {
            begin_index: self.begin_index,
            end_index: self.end_index,
            entries: MTSet::new(set2),
        }
    }
}