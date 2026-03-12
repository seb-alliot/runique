#[cfg(test)]
pub fn set_env(key: &str, val: &str) {
    unsafe { std::env::set_var(key, val) }
}

#[cfg(test)]
pub fn del_env(key: &str) {
    unsafe { std::env::remove_var(key) }
}
