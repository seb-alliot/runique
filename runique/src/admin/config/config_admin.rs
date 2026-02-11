// ═══════════════════════════════════════════════════════════════
// AdminConfig — Configuration du panneau d'administration
// ═══════════════════════════════════════════════════════════════

use std::sync::Arc;

use crate::middleware::auth::AdminAuth;

pub struct AdminConfig {
    /// Préfixe des routes admin (défaut : "/admin")
    pub prefix: String,

    /// Active le daemon de hot reload en développement
    pub hot_reload: bool,

    /// Titre affiché dans l'interface admin
    pub site_title: String,

    /// Active ou désactive entièrement l'AdminPanel
    pub enabled: bool,

    /// Handler de vérification du login admin
    ///
    /// Voir `crate::middleware::auth::AdminAuth`.
    pub auth: Option<Arc<dyn AdminAuth>>,
}

impl Clone for AdminConfig {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
            hot_reload: self.hot_reload,
            site_title: self.site_title.clone(),
            enabled: self.enabled,
            auth: self.auth.clone(),
        }
    }
}

impl std::fmt::Debug for AdminConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminConfig")
            .field("prefix", &self.prefix)
            .field("hot_reload", &self.hot_reload)
            .field("site_title", &self.site_title)
            .field("enabled", &self.enabled)
            .field("auth", &self.auth.as_ref().map(|_| "<AdminAuth>"))
            .finish()
    }
}

impl AdminConfig {
    pub fn new() -> Self {
        Self {
            prefix: "/admin".to_string(),
            hot_reload: false,
            site_title: "Administration".to_string(),
            enabled: true,
            auth: None,
        }
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload = enabled;
        self
    }

    pub fn site_title(mut self, title: &str) -> Self {
        self.site_title = title.to_string();
        self
    }

    /// Branche le handler d'authentification admin
    ///
    /// ```rust,ignore
    /// AdminConfig::new().auth(RuniqueAdminAuth::new())
    ///
    /// AdminConfig::new().auth(DefaultAdminAuth::<users::Entity>::new())
    /// ```
    pub fn auth<A: AdminAuth>(mut self, handler: A) -> Self {
        self.auth = Some(Arc::new(handler));
        self
    }

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
