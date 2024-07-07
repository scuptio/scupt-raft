use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use scupt_util::mt_set::MTSet;
use scupt_util::node_id::NID;
use serde::{Deserialize, Serialize};

use crate::msg_raft_state::MRaftState;
use crate::raft_message::{MAppendReq, MAppendResp, MApplyReq, MApplyResp, MDTMUpdateConfReq, MVoteReq, MVoteResp};

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
pub struct MUpdateConf {
    pub nid_vote: MTSet<NID>,
    pub nid_log: MTSet<NID>,
}

impl MsgTrait for MUpdateConf {}

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
pub enum MDTMTesting<T: MsgTrait + 'static> {
    #[serde(bound = "T: MsgTrait")]
    Setup(MRaftState<T>),
    #[serde(bound = "T: MsgTrait")]
    Check(MRaftState<T>),
    RequestVote,
    BecomeLeader,
    AppendLog,
    #[serde(bound = "T: MsgTrait")]
    ClientWriteLog(T),
    HandleVoteReq(MVoteReq),
    HandleVoteResp(MVoteResp),
    #[serde(bound = "T: MsgTrait")]
    HandleAppendReq(MAppendReq<T>),
    HandleAppendResp(MAppendResp),
    UpdateTerm(u64),
    Restart,
    LogCompaction(u64),
    AdvanceCommitIndex(u64),
    #[serde(bound = "T: MsgTrait")]
    HandleApplyReq(MApplyReq<T>),
    HandleApplyResp(MApplyResp),
    UpdateConfBegin(MUpdateConf),
    UpdateConfCommit,
    SendUpdateConf,
    UpdateConfReq(MDTMUpdateConfReq),
}

impl<T: MsgTrait + 'static> MsgTrait for MDTMTesting<T> {}