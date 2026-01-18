use crate::app_state::AppState;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// --- 1. Stockage temporaire pour la macro ---
// Ce buffer retient les URLs définies dans urlpatterns! le temps que l'App démarre.
pub static PENDING_URLS: Lazy<Mutex<Vec<(String, String)>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Utilisé par la macro urlpatterns!
pub fn register_pending(name: impl Into<String>, path: impl Into<String>) {
    let mut pending = PENDING_URLS.lock().unwrap();
    pending.push((name.into(), path.into()));
}

// --- 2. Fonctions utilisant l'AppState (Runtime) ---

pub fn register_name_url(state: &AppState, name: impl Into<String>, path: impl Into<String>) {
    let mut map = state.url_registry.write().unwrap();
    map.insert(name.into(), path.into());
}

pub fn reverse(state: &AppState, name: &str) -> Option<String> {
    let map = state.url_registry.read().unwrap();
    map.get(name).cloned()
}

pub fn reverse_with_parameters(
    state: &AppState,
    name: &str,
    parameters: &[(&str, &str)],
) -> Option<String> {
    let path = reverse(state, name)?;
    Some(
        parameters
            .iter()
            .fold(path, |acc, (k, v)| acc.replace(&format!("{{{}}}", k), v)),
    )
}

/// Transfère toutes les URLs en attente vers l'AppState
pub fn flush_pending_urls(state: &AppState) {
    let mut pending = PENDING_URLS.lock().unwrap();
    let mut map = state.url_registry.write().unwrap();

    for (name, path) in pending.drain(..) {
        map.insert(name, path);
    }
}
