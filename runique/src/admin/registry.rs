use crate::admin::resource::AdminResource;

#[derive(Debug, Default)]
pub struct AdminRegistry {
    pub resources: Vec<AdminResource>,
}

impl AdminRegistry {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Enregistre une nouvelle ressource
    ///
    /// Appelé par le code généré dans `target/runique/admin/generated.rs`
    pub fn register(&mut self, resource: AdminResource) {
        self.resources.push(resource);
    }

    /// Récupère une ressource par sa clé
    ///
    /// Ex: registry.get("users") → Some(&AdminResource { key: "users", ... })
    pub fn get(&self, key: &str) -> Option<&AdminResource> {
        self.resources.iter().find(|r| r.key == key)
    }

    /// Vérifie si une ressource est enregistrée
    pub fn contains(&self, key: &str) -> bool {
        self.resources.iter().any(|r| r.key == key)
    }

    /// Nombre de ressources enregistrées
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Vérifie si le registre est vide
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Retourne les clés de toutes les ressources
    pub fn keys(&self) -> Vec<&str> {
        self.resources.iter().map(|r| r.key).collect()
    }
}
