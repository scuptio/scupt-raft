#[cfg(test)]
mod tests {
    use scupt_util::logger::logger_setup;

    use crate::raft_message::RAFT;
    use crate::test_dtm::tests::{InputType, test_raft_gut};

    #[test]
    fn test_raft_1node() {
        logger_setup("debug");
        let path = "raft_trace_1n.db".to_string();
        test_raft_gut(InputType::FromDB(path.to_string()), 7555, 1, RAFT.to_string(), None)
    }
}