#![no_main]

use std::fs;
use libfuzzer_sys::fuzz_target;

use scupt_raft::test_quorum::tests::{
    _test_quorum_check_term_commit_index,
    TestParamCheckTermCommitIndex,
};

use scupt_raft::test_data::tests::{
    gen_test_json
};

use fuzz_raft::fuzz_gen_test_json::FUZZ_GEN_TEST_JSON;


fuzz_target!(|param:TestParamCheckTermCommitIndex| {
    let _p = param.clone();
    let ok = _test_quorum_check_term_commit_index(param);
    gen_test_json("/tmp/quorum_check_term_commit_index/", _p, ok, FUZZ_GEN_TEST_JSON);
});