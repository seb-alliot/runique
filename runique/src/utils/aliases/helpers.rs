use crate::utils::aliases::ARlockmap;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tower_http::services::ServeDir;

/// Crée un nouvel `Arc` autour d'une valeur.
///
/// # Exemple
///
/// ```rust
/// use std::sync::Arc;
/// let a = Arc::new(5);
/// assert_eq!(*a, 5);
/// ```
pub fn new<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

/// Crée un nouveau registre partagé (ARlockmap).
///
/// # Exemple
///
/// ```rust
/// use std::sync::{Arc, RwLock};
/// use std::collections::HashMap;
/// let registry: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
/// assert!(registry.read().unwrap().is_empty());
/// ```
pub fn new_registry() -> ARlockmap {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Crée un service ServeDir pour servir des fichiers statiques.
///
/// # Exemple
///
/// ```rust
/// use tower_http::services::ServeDir;
/// let serve = ServeDir::new("./static");
/// ```
pub fn new_serve<P: AsRef<std::path::Path>>(path: P) -> ServeDir {
    ServeDir::new(path)
}
