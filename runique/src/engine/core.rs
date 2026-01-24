use axum::{middleware, Router};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::Tera;

use crate::config::RuniqueConfig;
use crate::middleware::GardeFou;
use crate::middleware::{
    csrf_middleware, dev_no_cache_middleware, error_handler_middleware, sanitize_middleware,
    security_headers_middleware, CspConfig,
};
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
