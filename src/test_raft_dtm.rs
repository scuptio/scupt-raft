#[cfg(test)]
mod tests {
    use crate::test_dtm::tests::dtm_test_raft_n;

    #[test]
    fn test_raft_1n() {
        dtm_test_raft_n(1, 1000);
    }

    #[test]
    fn test_raft_2n() {
        dtm_test_raft_n(2, 1000);
    }

    #[test]
    fn test_raft_3n() {
        dtm_test_raft_n(3, 1000);
    }

    #[test]
    fn test_raft_5n() {
        dtm_test_raft_n(5, 1000);
    }
}