use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use serde::{Deserialize, Serialize};

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
pub struct TermIndex {
    pub term: u64,
    pub index: u64,
}

impl Default for TermIndex {
    fn default() -> Self {
        Self {
            term: 0,
            index: 0,
        }
    }
}

impl MsgTrait for TermIndex {}
