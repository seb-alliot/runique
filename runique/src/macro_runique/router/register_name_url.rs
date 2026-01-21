use crate::moteur_engine::engine_struct::RuniqueEngine;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

// --- 1. Stockage temporaire pour la macro ---
pub static PENDING_URLS: Lazy<Mutex<Vec<(String, String)>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Utilisé par la macro urlpatterns!
pub fn register_pending(name: impl Into<String>, path: impl Into<String>) {
    let mut pending = PENDING_URLS.lock().unwrap();
    pending.push((name.into(), path.into()));
}

// --- 2. Fonctions utilisant directement RuniqueEngine (Runtime) ---

/// Enregistre une URL dans l'engine
pub fn register_name_url(
    engine: &Arc<RuniqueEngine>,
    name: impl Into<String>,
    path: impl Into<String>,
) {
    let mut map = engine.url_registry.write().unwrap();
    map.insert(name.into(), path.into());
}

/// Récupère une URL à partir du nom
pub fn reverse(engine: &Arc<RuniqueEngine>, name: &str) -> Option<String> {
    let map = engine.url_registry.read().unwrap();
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
pub fn flush_pending_urls(engine: &Arc<RuniqueEngine>) {
    let mut pending = PENDING_URLS.lock().unwrap();
    let mut map = engine.url_registry.write().unwrap();

    for (name, path) in pending.drain(..) {
        map.insert(name, path);
    }
}
