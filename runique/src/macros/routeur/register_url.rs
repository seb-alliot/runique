//! Registre global des noms d'URL — `register_pending`, `reverse()`, `reverse_with_parameters()`.
use crate::engine::RuniqueEngine;
use std::sync::LazyLock;
use std::sync::{Arc, Mutex};

// --- 1. Stockage temporaire pour la macro ---
pub static PENDING_URLS: LazyLock<Mutex<Vec<(String, String)>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// Utilisé par la macro urlpatterns!
pub fn register_pending(name: impl Into<String>, path: impl Into<String>) {
    let mut pending = PENDING_URLS.lock().unwrap_or_else(|e| e.into_inner());
    pending.push((name.into(), path.into()));
}

// --- 2. Fonctions utilisant directement RuniqueEngine (Runtime) ---

/// Enregistre une URL dans l'engine
pub fn register_name_url(
    engine: &Arc<RuniqueEngine>,
    name: impl Into<String>,
    path: impl Into<String>,
) {
    let mut map = engine
        .url_registry
        .write()
        .unwrap_or_else(|e| e.into_inner());
    map.insert(name.into(), path.into());
}

/// Récupère une URL à partir du nom
pub fn reverse(engine: &Arc<RuniqueEngine>, name: &str) -> Option<String> {
    let map = engine
        .url_registry
        .read()
        .unwrap_or_else(|e| e.into_inner());
    map.get(name).cloned()
}

/// Récupère une URL avec substitution de paramètres
pub fn reverse_with_parameters(
    engine: &Arc<RuniqueEngine>,
    name: &str,
    parameters: &[(&str, &str)],
) -> Option<String> {
    let path = reverse(engine, name)?;
    let result = parameters
        .iter()
        .fold(path, |acc, (k, v)| acc.replace(&format!("{{{}}}", k), v));
    Some(result)
}

/// Transfère toutes les URLs en attente vers l'engine
pub fn add_urls(engine: &Arc<RuniqueEngine>) {
    let mut pending = PENDING_URLS.lock().unwrap_or_else(|e| e.into_inner());
    let mut map = engine
        .url_registry
        .write()
        .unwrap_or_else(|e| e.into_inner());

    for (name, path) in pending.drain(..) {
        map.insert(name, path);
    }
}
