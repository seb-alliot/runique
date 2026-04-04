//! Implémentation `FromRequestParts` pour `Request` — agrège engine, session, CSRF, flash en un seul extracteur.
use crate::context::Request;
use crate::flash::Message;
use crate::utils::aliases::AEngine;
use crate::utils::csrf::CsrfToken;
use axum::{extract::FromRequestParts, http::StatusCode, http::request::Parts};
use tower_sessions::Session;

/// Contexte principal pour un handler Runique
/// Contient :
// — L'engine principal (`RuniqueEngine`)
// — Le moteur de template (`TemplateEngine`)
// — Le gestionnaire de flash messages (`Message`)
pub struct RuniqueContext {
    pub engine: AEngine,
    pub tpl: Request,
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
        let engine = parts
            .extensions
            .get::<AEngine>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 2. Récupération de la session Tower
        let session = parts
            .extensions
            .get::<Session>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 3. Lecture du token CSRF injecté par le middleware CSRF
        // Si absent, le middleware n'est pas branché — erreur serveur
        let csrf_token = parts
            .extensions
            .get::<CsrfToken>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 4. Construction du contexte complet
        Ok(Self {
            engine: engine.clone(),
            tpl: Request::new(
                engine.clone(),
                session.clone(),
                csrf_token,
                parts.method.clone(),
            ),
            flash: Message {
                session: session.clone(),
            },
        })
    }
}
