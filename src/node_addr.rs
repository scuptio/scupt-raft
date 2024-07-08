use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use scupt_util::node_id::NID;
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
pub struct NodeAddr {
    pub node_id: NID,
    pub addr: String,
    pub port: u16,
}

impl MsgTrait for NodeAddr {}