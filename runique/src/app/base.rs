use crate::config::RuniqueConfig;
use crate::engine::RuniqueEngine;
use crate::middleware::MiddlewareConfig;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::Tera;

#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

/// Structure de base contenant toutes les dépendances communes
///
/// Cette struct centralise tout ce qui est partagé entre les différents builders
/// et évite la duplication de code lors de la construction de l'application.
///
/// # Champs
///
/// - `config` : Configuration de l'application (Arc pour partage)
/// - `tera` : Moteur de templates (Arc pour partage)
/// - `url_registry` : Registre des URLs nommées (RwLock pour modification)
/// - `middleware_config` : Configuration des middlewares
/// - `db` : Connexion à la base de données (optionnel avec feature "orm")
pub struct BaseApp {
    pub config: Arc<RuniqueConfig>,
    pub tera: Arc<Tera>,
    pub url_registry: Arc<RwLock<HashMap<String, String>>>,
    pub middleware_config: MiddlewareConfig,

    #[cfg(feature = "orm")]
    pub db: Arc<DatabaseConnection>,
}

impl BaseApp {
    /// Crée un Arc<RuniqueEngine> à partir de cette BaseApp
    ///
    /// Convertit la BaseApp en engine utilisable par les middlewares et handlers.
    ///
    /// # Exemple
    ///
    /// ```rust,no_run
    /// let base = create_base_app().await?;
    /// let engine = base.to_engine();
    /// ```
    pub fn to_engine(&self) -> Arc<RuniqueEngine> {
        Arc::new(RuniqueEngine {
            config: (*self.config).clone(),
            tera: self.tera.clone(),
            #[cfg(feature = "orm")]
            db: self.db.clone(),
            middleware_config: self.middleware_config.clone(),
            url_registry: self.url_registry.clone(),
            csp: Arc::new(Default::default()),
        })
    }

    /// Clone les références Arc pour usage dans les closures
    ///
    /// Utile pour les middlewares qui capturent les données.
    pub fn clone_refs(&self) -> (Arc<Tera>, Arc<RuniqueConfig>, Arc<RuniqueEngine>) {
        (self.tera.clone(), self.config.clone(), self.to_engine())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_app_to_engine() {
        // Test que la conversion vers engine fonctionne
        // (nécessite un setup complet, donc skip pour l'instant)
    }
}
