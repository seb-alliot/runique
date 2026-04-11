//! Registre central des ressources admin indexées par clé URL.
use indexmap::IndexMap;

use crate::admin::helper::resource_entry::ResourceEntry;
use crate::admin::resource::DisplayConfig;

/// Registre des ressources admin — IndexMap clé → ResourceEntry.
///
/// Alimenté par le code généré par le daemon (`src/admins/generated.rs`).
/// Partagé en lecture seule via `Arc<AdminRegistry>` dans l'état Axum.
/// L'ordre d'insertion (ordre dans `generated.rs`) est préservé.
#[derive(Default)]
pub struct AdminRegistry {
    pub resources: IndexMap<String, ResourceEntry>,
}

impl AdminRegistry {
    pub fn new() -> Self {
        Self {
            resources: IndexMap::new(),
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

    /// Applique une configuration d'affichage à une ressource existante (builtin ou déclarée).
    ///
    /// Appelé par le code généré après `admin_register()` pour les entrées du bloc `configure {}`.
    /// Sans effet si la clé n'existe pas.
    pub fn configure(&mut self, key: &str, display: DisplayConfig) {
        if let Some(entry) = self.resources.get_mut(key) {
            entry.meta.display = display;
        }
    }

    /// Réordonne le registre selon la liste de clés fournie.
    /// Les clés non listées sont ajoutées à la fin dans leur ordre d'insertion.
    pub fn reorder(&mut self, order: &[String]) {
        let mut reordered = indexmap::IndexMap::new();
        for key in order {
            if let Some(entry) = self.resources.shift_remove(key.as_str()) {
                reordered.insert(key.clone(), entry);
            }
        }
        // Clés restantes non listées
        for (key, entry) in std::mem::take(&mut self.resources) {
            reordered.insert(key, entry);
        }
        self.resources = reordered;
    }
}
