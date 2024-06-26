#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use scupt_util::logger::logger_setup;

    use crate::raft_message::RAFT;
    use crate::test_dtm::tests::{dtm_test_raft, InputType};
    use crate::test_path::tests::test_data_path;

    #[test]
    fn test_raft_all_input_from_db() {
        logger_setup("debug");
        for i in 1..=1000{
            let path = format!("raft_trace_2n_{}.db", i);
            let buf = PathBuf::from(test_data_path(path.clone()).unwrap());
            if !buf.exists() {
                break;
            }
            dtm_test_raft(InputType::FromDB(path),
                          2024, 2, RAFT.to_string(), None)
        }
    }
}