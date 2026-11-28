//! Lightweight `UrlParams` container for URL parameters (path + query) — unified and whitelisted access.
use crate::utils::aliases::StrMap;

/// Lightweight container for URL parameters (path + query).
/// Passed to form methods for whitelisted access.
pub struct UrlParams<'a> {
    pub path: &'a StrMap,
    pub query: &'a StrMap,
}

impl<'a> UrlParams<'a> {
    /// Wraps the given path and query maps for whitelisted lookup via [`get`](Self::get).
    pub fn new(path: &'a StrMap, query: &'a StrMap) -> Self {
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
