// ═══════════════════════════════════════════════════════════════
// AdminStaging — Configuration de l'AdminPanel dans le builder
// ═══════════════════════════════════════════════════════════════

use crate::admin::{AdminConfig, AdminRegistry};
use crate::app::error_build::{BuildError, CheckError, CheckReport};
use crate::middleware::auth::AdminAuth;

pub struct AdminStaging {
    pub(crate) config: AdminConfig,
    pub(crate) registry: AdminRegistry,
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

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.config = self.config.prefix(prefix);
        self
    }

    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.config = self.config.hot_reload(enabled);
        self
    }

    pub fn site_title(mut self, title: &str) -> Self {
        self.config = self.config.site_title(title);
        self
    }

    /// Branche le handler d'authentification admin
    ///
    /// ## Avec le User built-in (zéro config) :
    /// ```rust,ignore
    /// use runique::middleware::auth::RuniqueAdminAuth;
    ///
    /// .with_admin(|a| a
    ///     .site_title("Mon Admin")
    ///     .auth(RuniqueAdminAuth::new())
    /// )
    /// ```
    ///
    /// ## Avec un modèle custom :
    /// ```rust,ignore
    /// use runique::middleware::auth::{DefaultAdminAuth, UserEntity};
    ///
    /// impl UserEntity for users::Entity { ... }
    ///
    /// .with_admin(|a| a.auth(DefaultAdminAuth::<users::Entity>::new()))
    /// ```
    pub fn auth<A: AdminAuth>(mut self, handler: A) -> Self {
        self.config = self.config.auth(handler);
        self
    }

    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self.config = self.config.disable();
        self
    }

    pub(crate) fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

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

        if self.config.auth.is_none() {
            report.add(
                CheckError::new("AdminPanel", "Aucun handler d'authentification configuré")
                    .with_suggestion(
                        "Ajoutez .auth(RuniqueAdminAuth::new()) pour utiliser le User built-in, \
                     ou implémentez UserEntity sur votre propre modèle",
                    ),
            );
        }

        if report.has_errors() {
            return Err(BuildError::check(report));
        }

        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        if !self.enabled {
            return true;
        }
        !self.config.prefix.is_empty()
    }
}

impl Default for AdminStaging {
    fn default() -> Self {
        Self::new()
    }
}
