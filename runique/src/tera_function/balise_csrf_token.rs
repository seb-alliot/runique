use std::collections::HashMap;
use tera::{Function, Result as TeraResult, Value};

use async_trait::async_trait;
use axum::{
    body::Body, http::StatusCode, middleware::Next, response::IntoResponse, response::Response,
};
use tower_sessions::Session;
use tower_sessions::session::Error as SessionError;

use crate::middleware::csrf::CsrfToken;

/// Fonction Tera pour générer le champ CSRF
pub struct CsrfTokenFunction;
const CSRF_TOKEN_KEY: &str = "csrf_token";

impl Function for CsrfTokenFunction {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        // On récupère le token. S'il n'est pas passé en argument,
        // on peut le chercher dans le contexte global si ton système de rendu l'y met.
        let token = args.get("token").and_then(|v| v.as_str()).unwrap_or("");
        Ok(Value::String(format!(
            r#"<input type="hidden" name="csrf_token" value="{}">"#,
            token
        )))
    }
}

/// Enregistre la fonction csrf_token dans Tera
pub fn register_csrf_token(tera: &mut tera::Tera) {
    tera.register_function("csrf_token", CsrfTokenFunction);
}

#[async_trait]
pub trait SendCsrfToken {
    // Utilise un type d'erreur générique ou celui de tower_sessions
    async fn insert_csrf_token(&self, token: CsrfToken) -> Result<(), SessionError>;
}

#[async_trait]
impl SendCsrfToken for Session {
    async fn insert_csrf_token(&self, token: CsrfToken) -> Result<(), SessionError> {
        self.insert(CSRF_TOKEN_KEY, token).await
    }
}

/// Erreur de rejet pour l'Extractor. Nécessite IntoResponse pour qu'Axum la gère.
pub struct CsrfContextError(SessionError);
impl IntoResponse for CsrfContextError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur lors de la lecture du token CSRF: {}", self.0),
        )
            .into_response()
    }
}

pub async fn extract_csrf_token(mut req: axum::http::Request<Body>, next: Next) -> Response {
    let token = {
        // On utilise get_mut pour la session
        let session = match req.extensions().get::<Session>() {
            Some(s) => s,
            None => return next.run(req).await,
        };

        // On récupère le token
        session
            .get::<CsrfToken>(CSRF_TOKEN_KEY)
            .await
            .ok()
            .flatten()
    };

    if let Some(t) = token {
        req.extensions_mut().insert(t);
    }

    next.run(req).await
}
