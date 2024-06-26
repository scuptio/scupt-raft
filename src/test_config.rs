#[cfg(test)]
pub mod tests {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    pub static TEST_LOCK: Lazy<Mutex<()>> = Lazy::new(Mutex::default);
}



