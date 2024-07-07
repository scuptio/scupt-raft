use arbitrary::{Arbitrary, Unstructured};
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
pub struct NodeInfo {
    pub node_id: NID,
    pub can_vote: bool,
}

impl MsgTrait for NodeInfo {}


impl<'a> Arbitrary<'a> for NodeInfo {
    #[coverage(off)]
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            node_id: NID::arbitrary(u)?,
            can_vote: bool::arbitrary(u)?,
        })
    }
}