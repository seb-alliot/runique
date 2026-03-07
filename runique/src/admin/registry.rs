use std::collections::HashMap;

use crate::admin::resource_entry::ResourceEntry;

/// Registre des ressources admin — HashMap clé → ResourceEntry.
///
/// Alimenté par le code généré par le daemon (`src/admins/generated.rs`).
/// Partagé en lecture seule via `Arc<AdminRegistry>` dans l'état Axum.
#[derive(Default)]
pub struct AdminRegistry {
    pub resources: HashMap<String, ResourceEntry>,
}

impl AdminRegistry {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Enregistre une ressource. Appelé par le code généré au boot.
    pub fn register(&mut self, entry: ResourceEntry) {
        self.resources.insert(entry.meta.key.to_string(), entry);
    }

    /// Lookup par clé URL (ex: "users", "blog")
    pub fn get(&self, key: &str) -> Option<&ResourceEntry> {
        self.resources.get(key)
    }

    pub fn all(&self) -> impl Iterator<Item = &ResourceEntry> {
        self.resources.values()
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    pub fn len(&self) -> usize {
        self.resources.len()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.resources.contains_key(key)
    }

    pub fn keys(&self) -> Vec<&str> {
        self.resources.keys().map(|k| k.as_str()).collect()
    }
}
