use crate::middleware::login_requiert::is_authenticated;
use crate::settings::Settings;
use crate::utils::{generate_token, generate_user_token, mask_csrf_token, unmask_csrf_token};
use axum::{
    body::Body,
    http::{HeaderValue, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum::http::Request;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;

const CSRF_TOKEN_KEY: &str = "csrf_token";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CsrfToken(pub String);

pub async fn csrf_middleware(mut req: Request<Body>, next: Next) -> Response {
    // 0. Récupération config
    let config = match req.extensions().get::<Arc<Settings>>().cloned() {
        Some(c) => c,
        None => return next.run(req).await,
    };

    let method = req.method().clone();
    let requires_csrf = matches!(
        method,
        Method::POST | Method::PUT | Method::DELETE | Method::PATCH
    );

    // 1. Récupération session
    let session = match req.extensions().get::<Session>() {
        Some(s) => s.clone(),
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "Session missing").into_response(),
    };

    // 2. Récupérer ou créer le token
    let session_token = match session.get::<String>(CSRF_TOKEN_KEY).await.ok().flatten() {
        Some(t) => t,
        None => {
            let session_id = session.id().map(|id| id.to_string()).unwrap_or_default();

            // Détecter si utilisateur connecté
            let token = if is_authenticated(&session).await {
                let user_id = session.get::<i32>("user_id").await.ok().flatten().unwrap_or(0);
                generate_user_token(&config.server.secret_key, &user_id.to_string())
            } else {
                generate_token(&config.server.secret_key, &session_id)
            };

            let _ = session.insert(CSRF_TOKEN_KEY, token.clone()).await;
            token
        }
    };

    if requires_csrf {
        let content_type = req
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let mut request_token_masked = req
            .headers()
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        // Pour form-urlencoded, lire le body si pas de header
        if request_token_masked.is_none()
            && content_type.contains("application/x-www-form-urlencoded")
        {
            let (parts, body) = req.into_parts();
            let bytes = match body.collect().await {
                Ok(collected) => collected.to_bytes(),
                Err(_) => {
                    return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response();
                }
            };

            req = Request::from_parts(parts, Body::from(bytes.clone()));

            if let Ok(body_str) = std::str::from_utf8(&bytes) {
                request_token_masked = extract_csrf_from_form(body_str);
            }
        }

        let request_token = request_token_masked
            .as_deref()
            .and_then(|masked| unmask_csrf_token(masked).ok());

        match request_token {
            Some(rt) if constant_time_compare(&session_token, &rt) => {}
            _ => return (StatusCode::FORBIDDEN, "Invalid CSRF Token").into_response(),
        }
    }

    // 4. Injection du token masqué dans extensions + headers
    let masked = mask_csrf_token(&session_token);
    req.extensions_mut().insert(CsrfToken(masked.clone()));

    let mut response = next.run(req).await;
    if let Ok(hv) = HeaderValue::from_str(&masked) {
        response.headers_mut().insert("X-CSRF-Token", hv);
    }

    response
}

fn extract_csrf_from_form(body: &str) -> Option<String> {
    for pair in body.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            if key == "csrf_token" {
                return urlencoding::decode(value).ok().map(|s| s.to_string());
            }
        }
    }
    None
}

fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

#[async_trait::async_trait]
pub trait CsrfSession {
    async fn get_csrf_token(&self) -> Option<String>;
}

#[async_trait::async_trait]
impl CsrfSession for Session {
    async fn get_csrf_token(&self) -> Option<String> {
        self.get::<String>(CSRF_TOKEN_KEY).await.ok().flatten()
    }
}
