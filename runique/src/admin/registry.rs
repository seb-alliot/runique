//! Registre central des ressources admin indexées par clé URL.
use indexmap::IndexMap;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::admin::resource::DisplayConfig;
use crate::admin::resource_entry::ResourceEntry;

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

    /// Seed des droits scopés par ressource.
    ///
    /// Pour chaque ressource enregistrée, insère dans `eihwaz_droits` deux entrées si absentes :
    /// - `{key}.view` (`access_type = "view"`) — permission de voir la ressource dans la nav
    /// - `{key}.write` (`access_type = "write"`) — droit de modifier (create/edit/delete)
    ///
    /// Les ressources builtins superuser-only (`droits`, `groupes`) sont ignorées — elles n'ont
    /// pas de droits scopés : l'accès est contrôlé uniquement via `is_superuser`.
    pub async fn seed_resource_droits(&self, db: &DatabaseConnection) {
        use crate::admin::permissions::droit::{ActiveModel, Column, Entity};

        const SUPERUSER_ONLY: &[&str] = &["droits", "groupes"];

        for key in self.resources.keys() {
            if SUPERUSER_ONLY.contains(&key.as_str()) {
                continue;
            }

            for access_type in ["view", "write"] {
                let nom = format!("{}.{}", key, access_type);

                let exists = Entity::find()
                    .filter(Column::Nom.eq(&nom))
                    .one(db)
                    .await
                    .unwrap_or(None)
                    .is_some();

                if !exists {
                    let _ = ActiveModel {
                        nom: Set(nom),
                        resource_key: Set(Some(key.clone())),
                        access_type: Set(Some(access_type.to_string())),
                        ..Default::default()
                    }
                    .insert(db)
                    .await;
                }
            }
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
