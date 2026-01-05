use crate::middleware::flash_message::{FlashMessage, FlashMessageSession};
use crate::settings::Settings;
use crate::utils::{generate_token, mask_csrf_token, unmask_csrf_token};
use axum::{
    body::Body, http::HeaderValue, http::Method, http::StatusCode, middleware::Next,
    response::IntoResponse, response::Response,
};
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_sessions::Session;

const CSRF_TOKEN_KEY: &str = "csrf_token";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CsrfToken(pub String);

pub async fn csrf_middleware(mut req: axum::http::Request<Body>, next: Next) -> Response {
    let config = match req.extensions().get::<Arc<Settings>>().cloned() {
        Some(c) => c,
        None => return next.run(req).await,
    };

    let method = req.method().clone();
    let headers = req.headers().clone();
    let _uri_path = req.uri().path().to_string();

    let requires_csrf = matches!(
        method,
        Method::POST | Method::PUT | Method::DELETE | Method::PATCH
    );

    // Récupérer ou créer le token de session
    let session_token = {
        let session = match req.extensions().get::<Session>() {
            Some(s) => s,
            None => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Session middleware missing",
                )
                    .into_response();
            }
        };

        match session.get::<String>(CSRF_TOKEN_KEY).await.ok().flatten() {
            Some(token) => token,
            None => {
                let session_id = session
                    .id()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "no-session-id".to_string());
                let new_token = generate_token(&config.server.secret_key, &session_id);
                let _ = session.insert(CSRF_TOKEN_KEY, new_token.clone()).await;
                new_token
            }
        }
    };

    let token_to_inject: String = if requires_csrf {
        // Récupérer le token masqué
        let mut request_token_masked = headers
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok().map(|s| s.to_string()))
            .or_else(|| {
                headers
                    .get("X-CSRFToken")
                    .and_then(|h| h.to_str().ok().map(|s| s.to_string()))
            });

        // Lire le body
        let (parts, body) = req.into_parts();
        let bytes = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => {
                return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response();
            }
        };

        req = axum::http::Request::from_parts(parts, Body::from(bytes.clone()));
        req.extensions_mut().insert(bytes.clone());

        if let Ok(body_str) = std::str::from_utf8(&bytes) {
            if request_token_masked.is_none() {
                request_token_masked = extract_csrf_from_form(body_str);
            }
        }

        // Démasquer le token
        let request_token = request_token_masked
            .as_deref()
            .and_then(|masked| unmask_csrf_token(masked).ok());

        // Comparer les tokens
        match request_token {
            Some(rt) if constant_time_compare(&session_token, &rt) => {
                mask_csrf_token(&session_token)
            }
            _ => {
                let session = req.extensions().get::<Session>().cloned();
                if let Some(mut sess) = session {
                    let _ = sess
                        .insert_message(FlashMessage::error(
                            "Erreur de sécurité : Token CSRF invalide ou manquant.",
                        ))
                        .await;
                }

                if headers
                    .get("Accept")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("")
                    .contains("application/json")
                {
                    return (StatusCode::BAD_REQUEST, "Invalid CSRF Token").into_response();
                }

                return (StatusCode::FORBIDDEN, "Invalid CSRF Token").into_response();
            }
        }
    } else {
        mask_csrf_token(&session_token)
    };

    req.extensions_mut()
        .insert(CsrfToken(token_to_inject.clone()));

    let mut response = next.run(req).await;

    if let Ok(hv) = HeaderValue::from_str(&token_to_inject) {
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
