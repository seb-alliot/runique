use crate::context::request::{template, TemplateEngine};
use crate::engine::RuniqueEngine;
use crate::flash::Message;
use crate::middleware::is_authenticated;
use crate::utils::csrf::{CsrfContext, CsrfToken};
use axum::{extract::FromRequestParts, http::request::Parts, http::StatusCode};
use std::sync::Arc;
use tower_sessions::Session;

/// Contexte principal pour un handler Runique
/// Contient :
// — L'engine principal (`RuniqueEngine`)
// — Le moteur de template (`TemplateEngine`)
// — Le gestionnaire de flash messages (`Message`)
pub struct RuniqueContext {
    pub engine: Arc<RuniqueEngine>,
    pub tpl: template::TemplateEngine,
    pub flash: Message,
}

impl<S> FromRequestParts<S> for RuniqueContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    /// Constructeur de contexte depuis les extensions Axum
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Récupération de l'engine depuis les extensions
        // Injecté via un middleware lors du setup de l'application
        let engine = parts
            .extensions
            .get::<Arc<RuniqueEngine>>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 2. Récupération de la session Tower
        let session = parts
            .extensions
            .get::<Session>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 3. Récupération ou génération du token CSRF
        // On regarde d'abord si un token est déjà présent dans les extensions
        // Injecté par ton middleware CSRF
        let csrf_token: CsrfToken = if let Some(token) = parts.extensions.get::<CsrfToken>() {
            token.clone() // clone du token existant
        } else {
            // Génération d'un token selon que l'utilisateur est connecté ou non
            let session_id = "default_session_id"; // à remplacer par l'ID réel de session si disponible
            if is_authenticated(session).await {
                // Génère un token lié à l'utilisateur connecté
                CsrfToken::generate_with_context(
                    CsrfContext::Authenticated { user_id: 0 },
                    &engine.config.server.secret_key,
                )
            } else {
                // Génère un token lié à la session anonyme
                CsrfToken::generate_with_context(
                    CsrfContext::Anonymous { session_id },
                    &engine.config.server.secret_key,
                )
            }
        };
        println!(
            "CSRF Token Generated/Used extracteur ligne 44: {:?}",
            csrf_token
        );

        // 4. Construction du contexte complet
        Ok(Self {
            engine: engine.clone(),
            tpl: TemplateEngine::new(engine.clone(), csrf_token.masked().as_str().to_string()),
            flash: Message {
                session: session.clone(),
            }, // flash messages
        })
    }
}
