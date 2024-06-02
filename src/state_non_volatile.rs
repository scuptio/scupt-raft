use scupt_util::message::MsgTrait;

use crate::non_volatile_write::NonVolatileWrite;

pub struct StateNonVolatile<T: MsgTrait + 'static> {
    pub operation: Vec<NonVolatileWrite<T>>,
}


impl<T: MsgTrait + 'static> Default for StateNonVolatile<T> {
    fn default() -> Self {
        Self {
            operation: Default::default(),
        }
    }
}