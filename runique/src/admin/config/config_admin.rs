// ═══════════════════════════════════════════════════════════════
// AdminConfig — Configuration globale du panneau d'administration
// ═══════════════════════════════════════════════════════════════
//
// Centralise toutes les options de configuration de l'AdminPanel :
// - Préfixe des routes (/admin par défaut)
// - Hot reload (daemon actif en développement)
// - Titre du site admin (affiché dans l'interface)
// - Activation/désactivation complète de l'admin
//
// Configuré via AdminStaging dans le builder :
//   .with_admin(|a| a.prefix("/admin").hot_reload(true))
// ═══════════════════════════════════════════════════════════════

/// Configuration globale de l'AdminPanel
#[derive(Debug, Clone)]
pub struct AdminConfig {
    /// Préfixe des routes admin (défaut : "/admin")
    pub prefix: String,

    /// Active le daemon de hot reload (obligatoire pour le développement)
    ///
    /// Le daemon surveille `src/admin.rs` et régénère les handlers
    /// automatiquement à chaque modification.
    pub hot_reload: bool,

    /// Titre affiché dans l'interface d'administration
    pub site_title: String,

    /// Active ou désactive entièrement l'AdminPanel
    pub enabled: bool,
}

impl AdminConfig {
    /// Crée une configuration par défaut
    pub fn new() -> Self {
        Self {
            prefix: "/admin".to_string(),
            hot_reload: false,
            site_title: "Administration".to_string(),
            enabled: true,
        }
    }

    /// Donner la possibilité de choisir un préfixe pour la route admin
    ///
    /// ```rust,ignore
    /// AdminConfig::new().prefix("/backoffice")
    /// ```
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    /// Active ou désactive le hot reload du daemon
    ///
    /// Exemple idiomatique :
    /// ```rust,ignore
    /// .hot_reload(cfg!(debug_assertions)) // active en développement
    /// ```
    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload = enabled;
        self
    }

    /// Définit le titre affiché dans l'interface admin
    pub fn site_title(mut self, title: &str) -> Self {
        self.site_title = title.to_string();
        self
    }

    /// Désactive complètement l'AdminPanel
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self::new()
    }
}
