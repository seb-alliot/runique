//! Helper functions on aliases — `new()`, `new_registry()`, `new_serve()`.
use crate::utils::aliases::ARlockmap;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tower_http::services::ServeDir;

/// Creates a new `Arc` around a value.
#[doc = include_str!("../../../doc-tests/aliases/aliases_new.md")]
pub fn new<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

/// Creates a new shared registry (`ARlockmap`).
#[doc = include_str!("../../../doc-tests/aliases/aliases_new_registry.md")]
#[must_use]
pub fn new_registry() -> ARlockmap {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Creates a `ServeDir` service for serving static files.
///
/// # Exemple
///
/// ```rust,ignore
/// use tower_http::services::ServeDir;
/// let serve = ServeDir::new("./static");
/// ```
pub fn new_serve<P: AsRef<std::path::Path>>(path: P) -> ServeDir {
    ServeDir::new(path)
}
