use scupt_util::message::{Message, MsgTrait};

use crate::raft_message::RaftMessage;
use crate::state_non_volatile::StateNonVolatile;
use crate::state_volatile::StateVolatile;

pub struct RaftState<T: MsgTrait + 'static> {
    /// non-volatile states of the state machine
    pub non_volatile: StateNonVolatile<T>,
    /// volatile states of the state machine
    pub volatile: StateVolatile,
    /// message to send
    pub message: Vec<Message<RaftMessage<T>>>,

    pub _auto_name: String,
}

impl<T: MsgTrait + 'static> Default for RaftState<T> {
    fn default() -> Self {
        Self {
            non_volatile: Default::default(),
            volatile: Default::default(),
            message: vec![],
            _auto_name: "".to_string(),
        }
    }
}