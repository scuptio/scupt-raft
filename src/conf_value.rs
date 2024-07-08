use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
use scupt_util::node_id::NID;
use serde::{Deserialize, Serialize};

use crate::node_info::NodeInfo;

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
pub struct ConfValue {
    pub cluster_name: String,
    pub storage_path: String,
    pub node_id: NID,
    pub bind_address: String,
    pub bind_port: u16,
    pub timeout_max_tick: u64,
    pub millisecond_tick: u64,
    pub max_compact_entries: u64,
    pub send_value_to_leader: bool,
    pub node_peer: Vec<NodeInfo>,
}

impl MsgTrait for ConfValue {}


const MS_TIMEOUT_MAX_TICK: u64 = 500;

impl ConfValue {
    pub fn new(cluster_name: String,
               node_id: NID,
               storage_path: String,
               address: String,
               port: u16) -> Self {
        Self {
            cluster_name,
            storage_path,
            node_id,
            bind_address: address,
            bind_port: port,
            timeout_max_tick: MS_TIMEOUT_MAX_TICK,
            millisecond_tick: 50,
            max_compact_entries: 10,
            send_value_to_leader: false,
            node_peer: vec![],
        }
    }

    pub fn add_peers(&mut self, node_id: NID, can_vote: bool) {
        let peer = NodeInfo {
            node_id,

            can_vote,
        };
        self.node_peer.push(peer);
    }
}


impl Default for ConfValue {
    fn default() -> Self {
        Self {
            cluster_name: "".to_string(),
            storage_path: "".to_string(),
            node_id: 0,
            bind_address: "".to_string(),
            bind_port: 0,
            timeout_max_tick: 0,
            millisecond_tick: 0,
            max_compact_entries: 0,
            send_value_to_leader: false,
            node_peer: vec![],
        }
    }
}