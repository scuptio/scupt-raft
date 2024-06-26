use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;

use scupt_util::node_id::NID;
use serde::{Deserialize, Serialize};
use crate::conf_node::ConfNode;

use crate::conf_version::ConfVersion;
use crate::msg_dtm_testing::MDTMTesting;

use crate::raft_conf::ConfNodeValue;
use crate::snapshot::Snapshot;

pub const RAFT: &str = "Raft";
pub const RAFT_ABSTRACT: &str = "RAFT_ABSTRACT";
pub const RAFT_FUZZY: &str = "RAFT_FUZZY";


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
pub struct MVoteReq {
    pub term: u64,
    pub last_log_term: u64,
    pub last_log_index: u64,
}

impl MsgTrait for MVoteReq {}

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
pub struct MVoteResp {
    pub term: u64,
    pub vote_granted: bool,
}

impl MsgTrait for MVoteResp {}


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
pub struct PreVoteReq {
    pub source_nid: NID,
    pub request_term: u64,
    pub last_log_term: u64,
    pub last_log_index: u64,
}

impl MsgTrait for PreVoteReq {}

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
pub struct PreVoteResp {
    pub source_nid: NID,
    pub request_term: u64,
    pub vote_granted: bool,
}

impl MsgTrait for PreVoteResp {}

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
pub struct LogEntry<T: MsgTrait> {
    pub term: u64,
    pub index: u64,
    #[serde(bound = "T: MsgTrait")]
    pub value: T,
}

impl<T: MsgTrait + 'static> LogEntry<T> {
    pub fn map<T2, F>(&self, f: F) -> LogEntry<T2>
        where T2: MsgTrait + 'static,
              F: Fn(&T) -> T2
    {
        LogEntry {
            term: self.term,
            index: self.index,
            value: f(&self.value),
        }
    }
}

impl<T: MsgTrait + 'static> MsgTrait for LogEntry<T> {}

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
pub struct MAppendReq<T: MsgTrait + 'static> {
    pub term: u64,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    #[serde(bound = "T: MsgTrait")]
    pub log_entries: Vec<LogEntry<T>>,
    pub commit_index: u64,
}

impl<T: MsgTrait + 'static> MsgTrait for MAppendReq<T> {}

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
pub struct MAppendResp {
    pub term: u64,
    pub append_success: bool,
    pub commit_index: u64,
    pub match_index: u64,
    pub next_index: u64,
}

impl MsgTrait for MAppendResp {}

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
pub struct MApplyReq<T: MsgTrait + 'static> {
    pub term: u64,
    pub id: String,
    pub begin_index: u64,
    pub end_index: u64,
    #[serde(bound = "T: MsgTrait")]
    pub snapshot: Snapshot<T>,

}

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
pub struct MApplyResp {
    pub term: u64,
    pub match_index: u64,
    pub id: String,
}

impl<T: MsgTrait + 'static> MsgTrait for MApplyReq<T> {}

impl MsgTrait for MApplyResp {}


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
pub struct MClientReq<T: MsgTrait + 'static> {
    pub id: String,
    #[serde(bound = "T: MsgTrait")]
    pub value: T,
    pub source_id: Option<NID>,
    pub wait_write_local: bool,
    pub wait_commit: bool,
    pub from_client_request: bool,
}

impl<T: MsgTrait + 'static> MsgTrait for MClientReq<T> {}


pub const RCR_OK: u32 = 0;
pub const RCR_NOT_LEADER: u32 = 1;
pub const RCR_ERR_RESP: u32 = 2;

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
pub struct MClientResp {
    pub id: String,
    pub source_id: NID,
    pub index: u64,
    pub term: u64,
    pub error: u32,
    pub info: String,
}

impl MsgTrait for MClientResp {}

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
pub struct MUpdateConfReq {
    pub term: u64,
    pub conf_committed: ConfNodeValue,
    pub conf_new: ConfNodeValue,
}

impl MUpdateConfReq {
    pub(crate) fn to_dtm_msg(&self) -> MDTMUpdateConfReq {
        MDTMUpdateConfReq {
            term: self.term,
            conf_committed: self.conf_committed.node.clone(),
            conf_new: self.conf_new.node.clone(),
        }
    }
}
impl MsgTrait for MUpdateConfReq {}


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
pub struct MDTMUpdateConfReq {
    pub term: u64,
    pub conf_committed: ConfNode,
    pub conf_new: ConfNode,
}
impl MsgTrait for MDTMUpdateConfReq {

}
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
pub struct MUpdateConfResp {
    pub term: u64,
    // config term version committed
    pub conf_committed: ConfVersion,
    // new
    pub conf_new: ConfVersion,
}


impl MsgTrait for MUpdateConfResp {}

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
pub enum RaftMessage<T: MsgTrait + 'static> {
    PreVoteReq(PreVoteReq),
    PreVoteResp(PreVoteResp),
    VoteReq(MVoteReq),
    VoteResp(MVoteResp),
    #[serde(bound = "T: MsgTrait")]
    AppendReq(MAppendReq<T>),
    AppendResp(MAppendResp),
    #[serde(bound = "T: MsgTrait")]
    ApplyReq(MApplyReq<T>),
    #[serde(bound = "T: MsgTrait")]
    ApplyResp(MApplyResp),
    #[serde(bound = "T: MsgTrait")]
    ClientReq(MClientReq<T>),
    #[serde(bound = "T: MsgTrait")]
    ClientResp(MClientResp),
    UpdateConfReq(MUpdateConfReq),
    UpdateConfResp(MUpdateConfResp),
    #[serde(bound = "T: MsgTrait")]
    DTMTesting(MDTMTesting<T>), // only used when DTM testing
}

impl<T: MsgTrait + 'static> MsgTrait for RaftMessage<T> {}
