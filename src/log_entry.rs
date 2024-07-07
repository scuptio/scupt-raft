use bincode::{Decode, Encode};
use scupt_util::message::MsgTrait;
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
pub struct LogEntry<T: MsgTrait> {
    pub term: u64,
    pub index: u64,
    #[serde(bound = "T: MsgTrait")]
    pub value: T,
}

impl<T: MsgTrait + 'static> LogEntry<T> {
    pub fn map<T2, F>(&self, f: F) -> LogEntry<T2>
    where
        T2: MsgTrait + 'static,
        F: Fn(&T) -> T2,
    {
        LogEntry {
            term: self.term,
            index: self.index,
            value: f(&self.value),
        }
    }
}

impl<T: MsgTrait + 'static> MsgTrait for LogEntry<T> {}


#[cfg(test)]
mod test {
    use crate::log_entry::LogEntry;

    #[coverage(off)]
    #[test]
    fn test_log_entry() {
        let e = LogEntry {
            term: 0,
            index: 0,
            value: 100u64,
        };

        let e1 = e.map(|n| {
            n.to_string()
        });
        println!("{:?} {:?}", e, e1);
    }
}