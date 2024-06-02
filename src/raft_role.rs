use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use serde::{Deserialize, Serialize};

#[derive(
    Copy,
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
pub enum RaftRole {
    Leader,
    Follower,
    Candidate,
    Learner,
}

impl MsgTrait for RaftRole {}