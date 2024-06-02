use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use scupt_util::mt_set::MTSet;
use scupt_util::node_id::NID;
use serde::{Deserialize, Serialize};
use crate::conf_node::ConfNode;
use crate::conf_version::ConfVersion;
use crate::conf_value::ConfValue;

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
pub struct ConfNodeValue {
    pub node: ConfNode,
    pub value: ConfValue,
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
    Encode
)]
pub struct RaftConf {
    conf_committed: ConfNodeValue,
    conf_new: ConfNodeValue,
}

impl ConfNodeValue {
    pub fn from_conf_node_value(conf_node: ConfNode, conf_value: ConfValue) -> Self {
        Self {
            node: conf_node,
            value: conf_value,
        }
    }

    pub fn new(conf_value: ConfValue, term: u64, version: u64, commit_index: u64) -> Self {
        let mut nid_vote = vec![];
        let mut nid_log = vec![];

        for peer in conf_value.node_peer.iter() {
            if peer.can_vote {
                nid_vote.push(peer.node_id.clone());
            }
            nid_log.push(peer.node_id.clone());
        }
        let conf = ConfNodeValue {
            node: ConfNode {
                conf_version: ConfVersion { term, version, index: commit_index },
                nid_vote: MTSet::from_vec(nid_vote),
                nid_log: MTSet::from_vec(nid_log),
            },
            value: conf_value,
        };
        conf
    }

    pub fn nid_vote(&self) -> &Vec<NID> {
        self.node.nid_vote.vec()
    }
    pub fn term_version(&self) -> &ConfVersion {
        &self.node.conf_version
    }
}


impl Default for ConfNodeValue {
    fn default() -> Self {
        Self {
            node: Default::default(),
            value: Default::default(),
        }
    }
}

impl RaftConf {
    pub fn conf_node_value_new(&self) -> &ConfNodeValue {
        &self.conf_new
    }

    pub fn conf_node_value_committed(&self) -> &ConfNodeValue {
        &self.conf_committed
    }
    pub fn update_conf_committed(
        &mut self,
        conf: ConfNodeValue) {
        self.conf_committed = conf;
    }

    pub fn update_conf_new(
        &mut self,
        conf: ConfNodeValue) {
        self.conf_new = conf;
    }

    pub fn is_reconfig_ongoing(&self) -> bool {
        !self.conf_committed.term_version().eq(self.conf_new.term_version())
    }

    pub fn current_conf_value(&self) -> &ConfValue {
        &self.conf_committed.value
    }

    pub fn node_id(&self) -> NID {
        self.conf_committed.value.node_id
    }

    pub fn conf_node_committed(&self) -> &ConfNode {
        &self.conf_committed.node
    }

    pub fn conf_node_new(&self) -> &ConfNode {
        &self.conf_new.node
    }
    pub fn conf_committed_nid_vote(&self) -> &Vec<NID> {
        self.conf_committed.node.nid_vote.vec()
    }

    pub fn conf_new_nid_vote(&self) -> &Vec<NID> {
        self.conf_new.node.nid_vote.vec()
    }

    pub fn conf_nid_log(&self) -> &Vec<NID> {
        self.conf_committed.node.nid_log.vec()
    }

    pub fn conf_new_nid_log(&self) -> &Vec<NID> {
        self.conf_new.node.nid_log.vec()
    }
}

impl MsgTrait for RaftConf {}

impl Default for RaftConf {
    fn default() -> Self {
        Self {
            conf_committed: Default::default(),
            conf_new: Default::default(),
        }
    }
}