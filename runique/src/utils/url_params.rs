use std::collections::HashMap;

/// Conteneur léger pour les paramètres d'URL (path + query).
/// Passé aux méthodes de formulaire pour un accès whitelisté.
pub struct UrlParams<'a> {
    pub path: &'a HashMap<String, String>,
    pub query: &'a HashMap<String, String>,
}

impl<'a> UrlParams<'a> {
    pub fn new(path: &'a HashMap<String, String>, query: &'a HashMap<String, String>) -> Self {
        Self { path, query }
    }

    /// Cherche dans path d'abord, puis query.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.path
            .get(key)
            .or_else(|| self.query.get(key))
            .map(|s| s.as_str())
    }
}
