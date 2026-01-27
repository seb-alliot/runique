use crate::context::RequestExtensions;
use crate::engine::RuniqueEngine;
use crate::utils::csrf::{CsrfContext, CsrfToken};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Function, Result as TeraResult, Value};
use tower_sessions::Session;

pub struct CsrfTokenFunction;

impl Function for CsrfTokenFunction {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let token_str = args
            .get("csrf_token")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        Ok(Value::String(format!(
            r#"<input type="hidden" name="csrf_token" value="{}">"#,
            token_str
        )))
    }
}
const CSRF_TOKEN_KEY: &str = "csrf_token";

pub async fn csrf_middleware(
    State(engine): State<Arc<RuniqueEngine>>,
    session: Session,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let secret = &engine.config.server.secret_key;

    // Récupérer ou générer le token de session
    let session_token: CsrfToken = match session
        .get::<CsrfToken>(CSRF_TOKEN_KEY)
        .await
        .ok()
        .flatten()
    {
        Some(t) => {
            session
                .insert(CSRF_TOKEN_KEY, &t)
                .await
                .expect("Failed to re-insert CSRF token into session");
            t
        }
        None => {
            let token = if crate::middleware::auth::is_authenticated(&session).await {
                let user_id: i32 = session
                    .get::<i32>("user_id")
                    .await
                    .ok()
                    .flatten()
                    .unwrap_or(0);
                CsrfToken::generate_with_context(CsrfContext::Authenticated { user_id }, secret)
            } else {
                let session_id = session.id().map(|id| id.to_string()).unwrap_or_default();
                CsrfToken::generate_with_context(
                    CsrfContext::Anonymous {
                        session_id: &session_id,
                    },
                    secret,
                )
            };
            session
                .insert(CSRF_TOKEN_KEY, &token)
                .await
                .expect("Failed to insert CSRF token into session");
            token
        }
    };

    // Vérification CSRF si méthode sensible
    let requires_csrf = matches!(
        req.method(),
        &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH
    );

    if requires_csrf {
        let header_token = req
            .headers()
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok())
            .and_then(|masked| CsrfToken::unmasked(masked).ok());

        // Si le token est dans le header, on le vérifie
        if let Some(token) = header_token {
            if token.as_str() != session_token.as_str() {
                return (StatusCode::FORBIDDEN, "Invalid CSRF token (header)").into_response();
            }
        }
        // Sinon, on laisse Prisme vérifier le champ du formulaire
        // (la validation se fera dans is_valid())
    }

    // Injection du token masqué pour le frontend via la structure centralisée
    let masked = session_token.masked();
    let extensions = RequestExtensions::new().with_csrf_token(session_token.clone());

    extensions.inject_request(&mut req);

    let mut res = next.run(req).await;

    if let Ok(hv) = HeaderValue::from_str(masked.as_str()) {
        res.headers_mut().insert("X-CSRF-Token", hv);
    }

    res
}
