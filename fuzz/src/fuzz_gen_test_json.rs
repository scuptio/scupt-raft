#[cfg(_FUZZ_GEN_TEST_JSON_)]
pub const FUZZ_GEN_TEST_JSON: bool = true;

#[cfg(not(_FUZZ_GEN_TEST_JSON_))]
pub const FUZZ_GEN_TEST_JSON: bool = true;