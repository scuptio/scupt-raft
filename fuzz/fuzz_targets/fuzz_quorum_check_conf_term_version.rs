#![no_main]

use std::fs;
use libfuzzer_sys::fuzz_target;

use scupt_raft::test_quorum::tests::{
    _test_quorum_check_conf_term_version,
    TestParamCheckConfTermVersion,
};

use scupt_raft::test_data::tests::{
    gen_test_json
};

use fuzz_raft::fuzz_gen_test_json::FUZZ_GEN_TEST_JSON;


fuzz_target!(|param:TestParamCheckConfTermVersion| {
    let _p = param.clone();
    let ok = _test_quorum_check_conf_term_version(param);
    gen_test_json("/tmp/quorum_check_conf_term_version/", _p, ok, FUZZ_GEN_TEST_JSON);
});