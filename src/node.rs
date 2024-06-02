use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::node_addr::NodeAddr;
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
    Encode,
)]
struct Node {
    /// node address `[ip, port]` pairs, which would not be replicated
    pub node_addr: NodeAddr,
    /// node information that would be replicated to cluster
    pub node_info: NodeInfo,
}