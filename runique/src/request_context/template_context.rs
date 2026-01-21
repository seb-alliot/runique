// Déclaration des sous-modules

// Imports avec le nouveau chemin depuis la racine de src (crate::)
use crate::gardefou::composant_middleware::csrf::CsrfToken;
use crate::moteur_engine::engine_struct::RuniqueEngine;
use crate::request_context::composant_request::flash_struct::FlashManager;
use axum::response::IntoResponse;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use std::sync::Arc;
use tower_sessions::Session;

/// L'extracteur unique qui centralise l'accès à l'Engine, aux messages Flash et au CSRF
pub struct TemplateContext {
    pub engine: Arc<RuniqueEngine>,
    pub flash: FlashManager,
    pub csrf_token: String,
}

impl<S> FromRequestParts<S> for TemplateContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Récupération de l'Engine depuis les extensions (injecté via middleware)
        let engine = parts
            .extensions
            .get::<Arc<RuniqueEngine>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 2. Récupération du token CSRF depuis les extensions (injecté par ton middleware CSRF)
        let csrf_token: String = parts
            .extensions
            .get::<CsrfToken>()
            .map(|t| t.0.clone())
            .unwrap_or_default();

        // 3. Récupération de la Session DIRECTEMENT depuis Axum (tower_sessions l'ajoute automatiquement à Parts)
        // Et créer FlashManager à partir de la Session
        let session = parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let flash = FlashManager(session);

        Ok(Self {
            engine: engine.clone(),
            flash,
            csrf_token,
        })
    }
}

impl TemplateContext {
    /// Rend un template avec le contexte donné
    pub fn render(&self, template_name: &str, context: &tera::Context) -> axum::response::Response {
        // Cloner le contexte et ajouter les variables globales
        let mut ctx = context.clone();
        ctx.insert("debug", &self.engine.config.debug);
        ctx.insert("csrf_token", &self.csrf_token);

        match self.engine.tera.render(template_name, &ctx) {
            Ok(html) => axum::response::Html(html).into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur de template: {}", e),
            )
                .into_response(),
        }
    }

    /// Rend une page d'erreur 500
    pub fn render_500(&self, message: &str) -> axum::response::Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            message.to_string(),
        )
            .into_response()
    }
}
