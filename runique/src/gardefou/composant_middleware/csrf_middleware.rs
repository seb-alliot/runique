use crate::moteur_engine::engine_struct::RuniqueEngine;
use crate::utils::csrf::{CsrfContext, CsrfToken};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tower_sessions::Session;
use std::collections::HashMap;
use tera::{Function, Result as TeraResult, Value};

pub struct CsrfTokenFunction;

impl Function for CsrfTokenFunction {
    fn is_safe(&self) -> bool { true }
    
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

    // 1Récupérer ou générer le token de session
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
            let token = if crate::gardefou::composant_middleware::is_authenticated(&session).await {
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

    //  Vérification CSRF si méthode sensible
    let requires_csrf = matches!(
        req.method(),
        &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH
    );
    if requires_csrf {
        // Récupérer le token envoyé par le client (header)
        let request_token_masked = req
            .headers()
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok());
        // println!("Verication csrf middleware.rs ligne 50 CSRF token: session_token={}, request_token_masked={:?}", session_token.as_str(), request_token_masked);

        let request_token =
            request_token_masked.and_then(|masked| CsrfToken::unmasked(masked).ok());
        // println!("demaskage du csrf ligne 54 request token: {:?}", request_token.as_ref().map(|t| t.as_str()));

        match request_token {
            Some(req_t) if req_t.as_str() == session_token.as_str() => {}
            Some(_req_t) => {
                return (StatusCode::FORBIDDEN, "Invalid CSRF").into_response();
            }
            None => {
                return (StatusCode::FORBIDDEN, "Missing CSRF").into_response();
            }
        }
    } else {
        println!("Method does not require CSRF check");
    }

    // Injection du token masqué pour le frontend (template/AJAX)
    // Injection vers le frontend
    let masked = session_token.masked();
    req.extensions_mut().insert(session_token.clone());

    let mut res = next.run(req).await;

    if let Ok(hv) = HeaderValue::from_str(masked.as_str()) {
        res.headers_mut().insert("X-CSRF-Token", hv);
    }

    res
}
