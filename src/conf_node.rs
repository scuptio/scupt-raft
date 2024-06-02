use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use scupt_util::mt_set::MTSet;
use scupt_util::node_id::NID;
use serde::{Deserialize, Serialize};

use crate::conf_version::ConfVersion;

#[derive(

    Clone,
    Hash,
    PartialEq,
    Eq,
    Debug,
    Serialize,
    Deserialize,
    Decode,
    Encode
)]
pub struct ConfNode {
    pub conf_version: ConfVersion,
    /// node set which can vote, nid_vote is subset of nid_log
    pub nid_vote: MTSet<NID>,
    /// node set which can write log
    pub nid_log: MTSet<NID>,
}


impl ConfNode {
    pub fn conf_version(&self) -> &ConfVersion {
        &self.conf_version
    }
}


impl MsgTrait for ConfNode {}

impl Default for ConfNode {
    fn default() -> Self {
        Self {
            conf_version: Default::default(),
            nid_vote: Default::default(),
            nid_log: Default::default(),
        }
    }
}
