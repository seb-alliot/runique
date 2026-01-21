use axum::{middleware, Extension, Router};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::Tera;

use crate::gardefou::composant_middleware::{
    csrf::csrf_middleware, error_handler::error_handler_middleware,
    flash_message::flash_middleware, middleware_sanitiser::sanitize_middleware,
};

use crate::config_runique::config_struct::RuniqueConfig;
use crate::gardefou::composant_middleware::{csp::CspConfig, security_headers_middleware};
use crate::gardefou::middleware_struct::GardeFou;
#[cfg(feature = "orm")]
use sea_orm::DatabaseConnection;

/// Machine centrale de l'application
pub struct RuniqueEngine {
    pub config: RuniqueConfig,
    pub tera: Arc<Tera>,
    #[cfg(feature = "orm")]
    pub db: Arc<DatabaseConnection>,
    pub garde: GardeFou,
    pub url_registry: Arc<RwLock<HashMap<String, String>>>,
    pub csp: Arc<CspConfig>,
}

impl RuniqueEngine {
    pub fn attach_middlewares(&self, router: Router) -> Router {
        let engine = Arc::new(self.clone());
        router
            .layer(Extension(engine.clone()))
            .layer(middleware::from_fn_with_state(
                engine.clone(),
                sanitize_middleware,
            ))
            .layer(middleware::from_fn_with_state(
                engine.clone(),
                csrf_middleware,
            ))
            .layer(middleware::from_fn(flash_middleware))
            .layer(middleware::from_fn(error_handler_middleware))
            .layer(middleware::from_fn_with_state(
                engine.clone(),
                security_headers_middleware,
            ))
    }
}

// Pour pouvoir utiliser self.clone() dans les middlewares
impl Clone for RuniqueEngine {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            tera: Arc::clone(&self.tera),
            #[cfg(feature = "orm")]
            db: Arc::clone(&self.db),
            garde: self.garde.clone(),
            url_registry: Arc::clone(&self.url_registry),
            csp: Arc::clone(&self.csp),
        }
    }
}
