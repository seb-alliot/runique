//! Lightweight `UrlParams` container for URL parameters (path + query) — unified and whitelisted access.
use std::collections::HashMap;

/// Lightweight container for URL parameters (path + query).
/// Passed to form methods for whitelisted access.
pub struct UrlParams<'a> {
    pub path: &'a HashMap<String, String>,
    pub query: &'a HashMap<String, String>,
}

impl<'a> UrlParams<'a> {
    pub fn new(path: &'a HashMap<String, String>, query: &'a HashMap<String, String>) -> Self {
        Self { path, query }
    }

    /// Search in path first, then query.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.path
            .get(key)
            .or_else(|| self.query.get(key))
            .map(String::as_str)
    }
}
