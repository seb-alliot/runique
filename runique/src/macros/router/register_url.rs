//! Global registry of URL names — `register_pending`, `reverse()`, `reverse_with_parameters()`.
use crate::utils::aliases::{AEngine, ARlockmap, PendingUrls};
use std::sync::LazyLock;
use std::sync::Mutex;

// --- 1. Temporary storage for the macro ---
pub static PENDING_URLS: LazyLock<Mutex<PendingUrls>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// Used by the urlpatterns! macro
pub fn register_pending(name: impl Into<String>, path: impl Into<String>) {
    let mut pending = PENDING_URLS.lock().unwrap_or_else(|e| e.into_inner());
    pending.push((name.into(), path.into()));
}

// --- 2. Functions directly using RuniqueEngine (Runtime) ---

/// Registers a URL in the engine
pub fn register_name_url(engine: &AEngine, name: impl Into<String>, path: impl Into<String>) {
    let mut map = engine
        .url_registry
        .write()
        .unwrap_or_else(|e| e.into_inner());
    map.insert(name.into(), path.into());
}

/// Retrieves a URL from its name
pub fn reverse(engine: &AEngine, name: &str) -> Option<String> {
    let map = engine
        .url_registry
        .read()
        .unwrap_or_else(|e| e.into_inner());
    map.get(name).cloned()
}

/// Why a named URL could not be built.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum UrlError {
    #[error("Route '{0}' not found.\n\nVerify that the route exists in your routes!")]
    UnknownRoute(String),
    #[error("Route '{route}': missing parameter '{param}'")]
    MissingParam { route: String, param: String },
    #[error("Route '{route}': unknown parameter '{param}'")]
    UnknownParam { route: String, param: String },
}

/// Retrieves a URL with parameter substitution: every value is encoded, and the
/// given keys must match the route's placeholders exactly.
pub fn reverse_with_parameters(
    engine: &AEngine,
    name: &str,
    parameters: &[(&str, &str)],
) -> Result<String, UrlError> {
    resolve(&engine.url_registry, name, parameters)
}

/// Shared by `reverse_with_parameters` and the templates' `{% link %}`.
pub(crate) fn resolve(
    registry: &ARlockmap,
    name: &str,
    parameters: &[(&str, &str)],
) -> Result<String, UrlError> {
    let pattern = registry
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .get(name)
        .cloned()
        .ok_or_else(|| UrlError::UnknownRoute(name.to_string()))?;

    let mut used = vec![false; parameters.len()];
    let mut url = String::with_capacity(pattern.len());
    let mut rest = pattern.as_str();
    while let Some(start) = rest.find('{') {
        url.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            url.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let key = &after[..end];
        let index = parameters
            .iter()
            .position(|(k, _)| *k == key)
            .ok_or_else(|| UrlError::MissingParam {
                route: name.to_string(),
                param: key.to_string(),
            })?;
        used[index] = true;
        // A raw `/`, `?` or `#` in a value would change which route the URL reaches.
        url.push_str(&urlencoding::encode(parameters[index].1));
        rest = &after[end + 1..];
    }
    url.push_str(rest);

    if let Some(index) = used.iter().position(|u| !u) {
        return Err(UrlError::UnknownParam {
            route: name.to_string(),
            param: parameters[index].0.to_string(),
        });
    }
    Ok(url)
}

/// Transfers all pending URLs to the engine
pub fn add_urls(engine: &AEngine) {
    let mut pending = PENDING_URLS.lock().unwrap_or_else(|e| e.into_inner());
    let mut map = engine
        .url_registry
        .write()
        .unwrap_or_else(|e| e.into_inner());

    for (name, path) in pending.drain(..) {
        map.insert(name, path);
    }
}
