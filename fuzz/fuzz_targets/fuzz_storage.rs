#![no_main]

use std::fs;

use fuzz_raft::fuzz_gen_test_json::FUZZ_GEN_TEST_JSON;
use libfuzzer_sys::fuzz_target;
use scupt_raft::test_data::tests::gen_test_json;
use scupt_raft::test_storage::tests::{
    _test_storage,
    StorageOperation,
};

fuzz_target!(|param:StorageOperation| {
    let _p = param.clone();
    let _ = _test_storage(param);
});