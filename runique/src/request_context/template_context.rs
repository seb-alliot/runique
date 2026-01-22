use crate::moteur_engine::engine_struct::RuniqueEngine;
use crate::request_context::composant_request::flash_struct::FlashManager;
use crate::utils::{csp_nonce::CspNonce, csrf::CsrfToken};
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;
use tera::Context;
use tower_sessions::Session;
/// Contexte centralisé pour un handler Axum / template Tera
/// Contient :
/// - Engine (config, Tera, etc.)
/// - Flash messages
/// - Token CSRF
/// - Nonce CSP
/// - Helpers pour rendre les templates et injecter dynamiquement des variables

pub struct AppError(pub anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut res = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        // On injecte l'erreur réelle pour le middleware
        res.extensions_mut().insert(Arc::new(self.0));
        res
    }
}

pub struct TemplateContext {
    pub engine: Arc<RuniqueEngine>,
    pub messages: FlashManager,
    pub csrf_token: CsrfToken,
    pub csp_nonce: String,
    pub context: Context,
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
        let messages = FlashManager(session);

        //  Initialiser le contexte Tera avec les variables globales
        let mut context = Context::new();
        context.insert("debug", &engine.config.debug);
        context.insert("csrf_token", &csrf_token.masked().as_str());
        context.insert("csp_nonce", &csp_nonce);
        context.insert("static_runique", &engine.config.static_files);

        Ok(Self {
            engine: engine.clone(),
            messages,
            csrf_token,
            csp_nonce,
            context,
        })
    }
}

impl TemplateContext {
    /// Rendu d'un template Tera - Retourne un Result pour capturer la Stack Trace
    pub fn render(&self, template_name: &str) -> Result<Response, AppError> {
        match self.engine.tera.render(template_name, &self.context) {
            Ok(html) => Ok(Html(html).into_response()),
            Err(e) => {
                // On transforme l'erreur Tera en anyhow::Error pour capturer le contexte
                let err = anyhow::Error::new(e)
                    .context(format!("Failed to render template: {}", template_name));

                // On retourne l'AppError qui va remplir les extensions
                Err(AppError(err))
            }
        }
    }

    // Modifie aussi le helper context pour qu'il soit plus fluide avec le nouveau render
    pub fn context(&mut self, data: Vec<(&str, serde_json::Value)>) -> &mut Self {
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
