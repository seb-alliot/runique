use crate::request_context::composant_request;
use axum::{extract::FromRequestParts, http::request::Parts, http::StatusCode};
use std::sync::Arc;
use tower_sessions::Session;
use crate::moteur_engine::engine_struct::RuniqueEngine;
use crate::gardefou::composant_middleware::csrf::CsrfToken;

pub struct RuniqueContext {
    pub tpl: composant_request::template_struct::TemplateEngine,
    pub flash: composant_request::flash_struct::FlashManager,
}

impl<S> FromRequestParts<S> for RuniqueContext where S: Send + Sync {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let engine = parts.extensions.get::<Arc<RuniqueEngine>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        let session = parts.extensions.get::<Session>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        let csrf_token = parts.extensions.get::<CsrfToken>()
            .map(|t| t.0.clone()).unwrap_or_default();

        Ok(Self {
            tpl: composant_request::template_struct::TemplateEngine::new(engine.clone(), csrf_token),
            flash: composant_request::flash_struct::FlashManager(session.clone()),
        })
    }
}