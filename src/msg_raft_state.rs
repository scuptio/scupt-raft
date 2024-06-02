use std::fmt::Debug;
use std::hash::Hash;

use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use scupt_util::mt_map::MTMap;
use scupt_util::node_id::NID;
use serde::{Deserialize, Serialize};

use crate::conf_node::ConfNode;
use crate::conf_version::ConfVersionPair;
use crate::raft_message::LogEntry;
use crate::raft_role::RaftRole;
use crate::snapshot::Snapshot;
use crate::term_index::TermIndex;

/// raft state
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
pub struct MRaftState<T: MsgTrait + 'static> {
    pub role: RaftRole,
    pub current_term: u64,
    #[serde(bound = "T: MsgTrait")]
    pub log: Vec<LogEntry<T>>,
    #[serde(bound = "T: MsgTrait")]
    pub snapshot: Snapshot<T>,
    pub voted_for: Option<NID>,
    pub commit_index: u64,
    pub conf_committed: ConfNode,
    pub conf_new: ConfNode,
    pub follower_vote_granted: MTMap<NID, u64>,
    pub follower_next_index: MTMap<NID, u64>,
    pub follower_match_index: MTMap<NID, u64>,
    pub follower_term_commit_index: MTMap<NID, TermIndex>,
    pub follower_conf: MTMap<NID, ConfVersionPair>,
}

impl<T: MsgTrait + 'static> MsgTrait for MRaftState<T> {}

