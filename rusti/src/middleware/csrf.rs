use tower_sessions::Session;
use axum::{
    response::Response,
    http::StatusCode,
    middleware::Next,
    body::Body,
    http::Method,
    response::IntoResponse,
};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use std::sync::Arc;
use crate::settings::Settings;
use serde::{Serialize, Deserialize};
type HmacSha256 = Hmac<Sha256>;

const CSRF_TOKEN_KEY: &str = "csrf_token";
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CsrfToken(pub String);

pub fn generate_csrf_token(secret_key: &str, session_id: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");

    mac.update(b"rusti.middleware.csrf");
    mac.update(session_id.as_bytes());

    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

pub async fn csrf_middleware(
    mut req: axum::http::Request<Body>,
    next: Next,
) -> Response {
    let config_opt = req.extensions().get::<Arc<Settings>>().cloned();
    let method = req.method().clone();
    let headers = req.headers().clone();

    // Si pas de config, passer directement
    let config = match config_opt {
        Some(c) => c,
        None => return next.run(req).await,
    };

    // Vérifier le CSRF
    let should_block: Option<Response> = {
        let session = match req.extensions_mut().get_mut::<Session>() {
            Some(s) => s,
            None => return next.run(req).await,
        };

        let requires_csrf = matches!(
            &method,
            &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH
        );

        if requires_csrf {
            let session_token = session
                .get::<String>(CSRF_TOKEN_KEY)
                .await
                .ok()
                .flatten();

            let request_token = headers
                .get("X-CSRF-Token")
                .and_then(|h| h.to_str().ok())
                .or_else(|| {
                    headers.get("X-CSRFToken").and_then(|h| h.to_str().ok())
                });

            match (session_token, request_token) {
                (Some(st), Some(rt)) if constant_time_compare(&st, rt) => None,
                _ => {
                    tracing::error!("CSRF verification failed");
                    Some(if config.debug {
                        (StatusCode::FORBIDDEN, "CSRF token verification failed").into_response()
                    } else {
                        (StatusCode::FORBIDDEN, "Forbidden").into_response()
                    })
                }
            }
            } else {
                        // 1. On récupère le token existant en session
                        let existing_token = session.get::<String>(CSRF_TOKEN_KEY).await.ok().flatten();

                        let token = if let Some(t) = existing_token {
                            t // Utilise le token existant
                        } else {
                            // 2. Ou on en génère un nouveau si absent
                            let session_id = session.id()
                                .map(|id| id.to_string())
                                .unwrap_or_else(|| "no-session-id".to_string());

                            let new_token = generate_csrf_token(&config.server.secret_key, &session_id);
                            let _ = session.insert(CSRF_TOKEN_KEY, new_token.clone()).await;
                            new_token
                        };
            req.extensions_mut().insert(CsrfToken(token));
            None
        }
    };
    if let Some(error_response) = should_block {
        error_response
    } else {
        next.run(req).await
    }
}

fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();

    let mut result = 0u8;
    for i in 0..a_bytes.len() {
        result |= a_bytes[i] ^ b_bytes[i];
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


