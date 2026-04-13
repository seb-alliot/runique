//! `FromRequestParts` implementation for `Request` — aggregates engine, session, CSRF, and flash into a single extractor.
use crate::context::Request;
use crate::flash::Message;
use crate::utils::aliases::AEngine;
use crate::utils::csrf::CsrfToken;
use axum::{extract::FromRequestParts, http::StatusCode, http::request::Parts};
use tower_sessions::Session;

/// Main context for a Runique handler
/// Contains:
// — The main engine (`RuniqueEngine`)
// — The template engine (`TemplateEngine`)
// — The flash messages manager (`Message`)
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

    /// Context constructor from Axum extensions
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Retrieving the engine from extensions
        let engine = parts
            .extensions
            .get::<AEngine>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 2. Retrieving the Tower session
        let session = parts
            .extensions
            .get::<Session>()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 3. Reading the CSRF token injected by the CSRF middleware
        // If absent, the middleware is not attached — server error
        let csrf_token = parts
            .extensions
            .get::<CsrfToken>()
            .cloned()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 4. Building the complete context
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
