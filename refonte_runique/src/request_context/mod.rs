// Déclaration des sous-modules
pub mod composant_request;
pub mod tera_tool;
pub mod request_struct;

// Imports avec le nouveau chemin depuis la racine de src (crate::)
use crate::moteur_engine::engine_struct::RuniqueEngine;
use crate::request_context::composant_request::flash_struct::FlashManager;
use crate::gardefou::composant_middleware::csrf::CsrfToken;

use std::sync::Arc;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

/// L'extracteur unique qui centralise l'accès à l'Engine, aux messages Flash et au CSRF
pub struct RuniqueContext {
    pub engine: Arc<RuniqueEngine>,
    pub flash: FlashManager,
    pub csrf_token: String,
}

impl<S> FromRequestParts<S> for RuniqueContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Récupération de l'Engine depuis les extensions (injecté via .layer(Extension(engine)))
        let engine = parts.extensions.get::<Arc<RuniqueEngine>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 2. Récupération du token CSRF depuis les extensions (injecté par ton middleware CSRF)
        let csrf_token = parts.extensions.get::<CsrfToken>()
            .map(|t| t.0.clone())
            .unwrap_or_default();

        // 3. Récupération des messages Flash (injectés par ton flash_middleware)
        let flash = parts.extensions.get::<FlashManager>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        
        Ok(Self {
            engine: engine.clone(),
            flash,
            csrf_token,
        })
    }
}