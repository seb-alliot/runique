#[cfg(test)]
pub fn set_env(key: &str, val: &str) {
    unsafe { std::env::set_var(key, val) }
}

#[cfg(test)]
pub fn del_env(key: &str) {
    unsafe { std::env::remove_var(key) }
}

/// Puts back the test run's `MEDIA_ROOT` (`runique/.cargo/config.toml`) rather
/// than removing it: a test building an app meanwhile would create
/// `{cwd}/media`. Removed when the run sets none.
#[cfg(test)]
pub fn restore_media_root() {
    match option_env!("MEDIA_ROOT") {
        Some(root) => set_env("MEDIA_ROOT", root),
        None => del_env("MEDIA_ROOT"),
    }
}
