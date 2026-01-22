use crate::utils::csrf::{CsrfToken, CsrfContext};
use crate::moteur_engine::engine_struct::RuniqueEngine;
use axum::{
    body::Body,
    http::{HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    extract::State,
};
use std::sync::Arc;
use tower_sessions::Session;

const CSRF_TOKEN_KEY: &str = "csrf_token";

pub async fn csrf_middleware(
    State(engine): State<Arc<RuniqueEngine>>,
    session: Session,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let secret = &engine.config.server.secret_key;

    println!("--- CSRF Middleware START ---");
    println!("Request method: {:?}", req.method());

    // 1️⃣ Récupérer ou générer le token de session
    let session_token: CsrfToken = match session.get::<CsrfToken>(CSRF_TOKEN_KEY).await.ok().flatten() {
        Some(t) => {
            session.insert(CSRF_TOKEN_KEY, &t).await.expect("Failed to re-insert CSRF token into session");
            println!("Found existing session token: {}", t.as_str());
            t
        },
        None => {
            println!("No CSRF token in session, generating a new one...");
            let token = if crate::gardefou::composant_middleware::is_authenticated(&session).await {
                let user_id: i32 = session.get::<i32>("user_id").await.ok().flatten().unwrap_or(0);
                println!("User is authenticated, user_id = {}", user_id);
                CsrfToken::generate_with_context(CsrfContext::Authenticated { user_id }, secret)
            } else {
                let session_id = session.id().map(|id| id.to_string()).unwrap_or_default();
                println!("User is anonymous, session_id = {}", session_id);
                CsrfToken::generate_with_context(CsrfContext::Anonymous { session_id: &session_id }, secret)
            };
            session.insert(CSRF_TOKEN_KEY, &token).await.expect("Failed to insert CSRF token into session");
            token
        }
    };

    // 2️⃣ Vérification CSRF si méthode sensible
    let requires_csrf = matches!(req.method(), &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH);
    if requires_csrf {
        println!("Method requires CSRF check");

        // Récupérer le token envoyé par le client (header)
        let request_token_masked = req.headers()
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok());

        let request_token = request_token_masked
            .and_then(|masked| CsrfToken::unmasked(masked).ok());

        match request_token {
            Some(req_t) if req_t.as_str() == session_token.as_str() => {
                println!("CSRF valid ✅");
            }
            Some(req_t) => {
                println!("CSRF mismatch ❌ expected {}, got {}", session_token.as_str(), req_t.as_str());
                return (StatusCode::FORBIDDEN, "Invalid CSRF").into_response();
            }
            None => {
                println!("CSRF missing ❌ expected {}", session_token.as_str());
                return (StatusCode::FORBIDDEN, "Missing CSRF").into_response();
            }
        }

    } else {
        println!("Method does not require CSRF check");
    }

    // 3️⃣ Injection du token masqué pour le frontend (template/AJAX)
    // Injection vers le frontend
    let masked = session_token.masked();
    req.extensions_mut().insert(masked.clone());
    println!("Injected masked token: {}", masked.as_str());


    let mut res = next.run(req).await;

    if let Ok(hv) = HeaderValue::from_str(masked.as_str()) {
        res.headers_mut().insert("X-CSRF-Token", hv);
        println!("Added masked CSRF token to response headers ligne 92");
    }

    println!("--- CSRF Middleware END ---");
    res
}
