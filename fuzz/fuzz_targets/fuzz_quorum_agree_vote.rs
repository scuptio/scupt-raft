#![no_main]

use libfuzzer_sys::fuzz_target;
use scupt_raft::test_quorum::tests::{
    _test_quorum_agree_vote,
    TestParamAgreeVote,
};
use scupt_raft::test_data::tests::{
    gen_test_json
};
use fuzz_raft::fuzz_gen_test_json::FUZZ_GEN_TEST_JSON;

fuzz_target!(|param:TestParamAgreeVote| {
    let _p = param.clone();
    let index = _test_quorum_agree_vote(param);
    gen_test_json("/tmp/quorum_agree_vote/", _p, index, FUZZ_GEN_TEST_JSON);
});
