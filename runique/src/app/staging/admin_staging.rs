// ═══════════════════════════════════════════════════════════════
// AdminStaging — Configuration de l'AdminPanel dans le builder
// ═══════════════════════════════════════════════════════════════
//
// Même pattern que CoreStaging, MiddlewareStaging, StaticStaging.
// Collecte flexible → validation → construction dans build().
//
// Usage :
//   RuniqueApp::builder(config)
//       .routes(url::routes())
//       .with_database(db)
//       .with_admin(|a| a
//           .prefix("/admin")
//           .hot_reload(cfg!(debug_assertions))
//           .site_title("Mon Admin")
//       )
//       .build().await?
// ═══════════════════════════════════════════════════════════════

use crate::admin::{AdminConfig, AdminRegistry};
use crate::app::error_build::{BuildError, CheckError, CheckReport};

pub struct AdminStaging {
    /// Configuration admin (préfixe, hot_reload, titre...)
    pub(crate) config: AdminConfig,

    /// Registre des ressources (rempli par le code généré)
    pub(crate) registry: AdminRegistry,

    /// AdminPanel activé ou non
    pub(crate) enabled: bool,
}

impl AdminStaging {
    pub fn new() -> Self {
        Self {
            config: AdminConfig::new(),
            registry: AdminRegistry::new(),
            enabled: false,
        }
    }

    /// Définit le préfixe des routes admin (défaut : "/admin")
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.config = self.config.prefix(prefix);
        self
    }

    /// Active ou désactive le hot reload du daemon
    ///
    /// ```rust,ignore
    /// .with_admin(|a| a.hot_reload(cfg!(debug_assertions)))
    /// ```
    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.config = self.config.hot_reload(enabled);
        self
    }

    /// Définit le titre affiché dans l'interface admin
    pub fn site_title(mut self, title: &str) -> Self {
        self.config = self.config.site_title(title);
        self
    }

    /// Désactive complètement l'AdminPanel
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self.config = self.config.disable();
        self
    }

    /// Active l'AdminPanel (appelé automatiquement par `.with_admin()`)
    pub(crate) fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Injecte un registre de ressources (appelé par le code généré)
    pub fn with_registry(mut self, registry: AdminRegistry) -> Self {
        self.registry = registry;
        self
    }

    pub fn validate(&self) -> Result<(), BuildError> {
        if !self.enabled {
            return Ok(());
        }

        let mut report = CheckReport::new();

        if self.config.prefix.is_empty() {
            report.add(
                CheckError::new(
                    "AdminPanel",
                    "Le préfixe des routes admin ne peut pas être vide",
                )
                .with_suggestion("Utilisez .prefix(\"/admin\") ou laissez la valeur par défaut"),
            );
        }

        if report.has_errors() {
            return Err(BuildError::check(report));
        }

        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        if !self.enabled {
            return true; // Désactivé = pas bloquant
        }
        !self.config.prefix.is_empty()
    }
}

impl Default for AdminStaging {
    fn default() -> Self {
        Self::new()
    }
}
