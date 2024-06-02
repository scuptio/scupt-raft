#[cfg(test)]
mod tests {
    use scupt_util::logger::logger_setup;

    use crate::raft_message::RAFT;
    use crate::test_dtm::tests::{InputType, dtm_test_raft};

    #[test]
    fn test_raft_1node() {
        logger_setup("debug");
        let path = "raft_trace_1n.db".to_string();
        dtm_test_raft(InputType::FromDB(path.to_string()), 7555, 1, RAFT.to_string(), None)
    }
}