use crate::moteur_engine::engine_struct::RuniqueEngine;
use crate::request_context::composant_request::flash_struct::FlashManager;
use crate::utils::{csrf::CsrfToken, csp_nonce::CspNonce};
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;
use tower_sessions::Session;
use tera::Context;
use serde_json::Value;
/// Contexte centralisé pour un handler Axum / template Tera
/// Contient :
/// - Engine (config, Tera, etc.)
/// - Flash messages
/// - Token CSRF
/// - Nonce CSP
/// - Helpers pour rendre les templates et injecter dynamiquement des variables
pub struct TemplateContext {
    pub engine: Arc<RuniqueEngine>,
    pub flash: FlashManager,
    pub csrf_token: CsrfToken,
    pub csp_nonce: String,
    pub context: Context, // contexte Tera interne pour le rendu
}

impl<S> FromRequestParts<S> for TemplateContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        //  Récupération de l'Engine depuis les extensions
        let engine = parts
            .extensions
            .get::<Arc<RuniqueEngine>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        //  Récupération du token CSRF depuis les extensions
        let csrf_token: CsrfToken = parts
            .extensions
            .get::<CsrfToken>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        //  Récupération du nonce CSP depuis les extensions
        let csp_nonce: String = parts
            .extensions
            .get::<CspNonce>()
            .map(|n| n.as_str().to_string())
            .unwrap_or_default();

        //  Récupération de la session et création du FlashManager
        let session = parts
            .extensions
            .get::<Session>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
        let flash = FlashManager(session);

        //  Initialiser le contexte Tera avec les variables globales
        let mut context = Context::new();
        context.insert("debug", &engine.config.debug);
        context.insert("csrf_token", &csrf_token.masked().as_str());
        println!("Injected CSRF token into TemplateContext: {:?}", csrf_token.as_str());
        context.insert("csp_nonce", &csp_nonce);
        context.insert("static_runique", &engine.config.static_files);

        Ok(Self {
            engine: engine.clone(),
            flash,
            csrf_token,
            csp_nonce,
            context,
        })
    }
}

impl TemplateContext {
    /// Rendu d'un template Tera avec le contexte actuel
    pub fn render(&self, template_name: &str) -> Response {
        match self.engine.tera.render(template_name, &self.context) {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                tracing::error!("Erreur de rendu Tera: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Erreur de rendu interne").into_response()
            }
        }
    }

    /// Helper pour insérer dynamiquement des variables dans le template
    pub fn context_update(&mut self, data: Vec<(&str, Value)>) -> &mut Self {
        for (key, value) in data {
            self.context.insert(key, &value);
        }
        self
    }

    /// Rendu d'une page 500 avec message custom
    pub fn render_500(&self, message: &str) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, message.to_string()).into_response()
    }
}
