use axum::{middleware, Router};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::Tera;

use crate::gardefou::composant_middleware::{
    csrf_middleware::csrf_middleware, dev_cache::dev_no_cache_middleware,
    error_handler::error_handler_middleware, middleware_sanitiser::sanitize_middleware,
};

use crate::config_runique::config_struct::RuniqueConfig;
use crate::gardefou::composant_middleware::{
    csp_middleware::CspConfig, security_headers_middleware,
};
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
    pub fn attach_middlewares(engine: Arc<Self>, router: Router) -> Router {
        router
            .layer(middleware::from_fn_with_state(
                engine.clone(),
                sanitize_middleware,
            ))
            .layer(middleware::from_fn_with_state(
                engine.clone(),
                csrf_middleware,
            ))
            .layer(middleware::from_fn_with_state(
                engine.clone(),
                dev_no_cache_middleware,
            ))
            .layer(middleware::from_fn(error_handler_middleware))
            .layer(middleware::from_fn_with_state(
                engine.clone(),
                security_headers_middleware,
            ))
    }
}
