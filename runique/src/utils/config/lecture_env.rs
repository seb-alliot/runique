pub fn env_or_default(var: &str, default: &str) -> String {
    std::env::var(var).unwrap_or(default.to_string())
}
