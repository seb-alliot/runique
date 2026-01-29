use crate::utils::aliases::ARlockmap;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tower_http::services::ServeDir;

pub fn new<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

pub fn new_registry() -> ARlockmap {
    Arc::new(RwLock::new(HashMap::new()))
}

pub fn new_serve<P: AsRef<std::path::Path>>(path: P) -> ServeDir {
    ServeDir::new(path)
}
