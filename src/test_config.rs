#[cfg(test)]
pub mod tests {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    pub const SECONDS_TEST_RUN_MAX: u64 = 300u64;

    pub static TEST_LOCK: Lazy<Mutex<()>> = Lazy::new(Mutex::default);
}



