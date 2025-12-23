use tower_sessions::Session;
use axum::{
    response::Response,
    http::StatusCode,
    middleware::Next,
    body::Body,
    http::Method,
    response::IntoResponse,
    http::HeaderValue,
    response::Redirect,
};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use std::sync::Arc;
use crate::settings::Settings;
use serde::{Serialize, Deserialize};
use http_body_util::BodyExt;
use crate::middleware::flash_message::{FlashMessage, FlashMessageSession};

type HmacSha256 = Hmac<Sha256>;

const CSRF_TOKEN_KEY: &str = "csrf_token";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CsrfToken(pub String);

/// Génère un token CSRF unique en utilisant session_id + timestamp
pub fn generate_csrf_token(secret_key: &str, session_id: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");

    mac.update(b"rusti.middleware.csrf");
    mac.update(session_id.as_bytes());

    // 🔥 Ajout d'un timestamp pour rendre chaque token unique
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();
    mac.update(timestamp.as_bytes());

    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

pub async fn csrf_middleware(
    mut req: axum::http::Request<Body>,
    next: Next,
) -> Response {
    let config = match req.extensions().get::<Arc<Settings>>().cloned() {
        Some(c) => c,
        None => return next.run(req).await,
    };

    let method = req.method().clone();
    let headers = req.headers().clone();
    let uri_path = req.uri().path().to_string();

    let requires_csrf = matches!(
        method,
        Method::POST | Method::PUT | Method::DELETE | Method::PATCH
    );

    let token_to_inject: String = if requires_csrf {
        // 1. Récupérer le token de session
        let session_token = {
            let session = match req.extensions().get::<Session>() {
                Some(s) => s,
                None => return (StatusCode::INTERNAL_SERVER_ERROR, "Session middleware missing").into_response(),
            };
            let t = session.get::<String>(CSRF_TOKEN_KEY).await.ok().flatten();
            t
        };

        // 2. Récupérer le token de la requête
        let mut request_token = headers
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok().map(|s| s.to_string()))
            .or_else(|| {
                headers.get("X-CSRFToken")
                    .and_then(|h| h.to_str().ok().map(|s| s.to_string()))
            });

        let (parts, body) = req.into_parts();
        let bytes = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(_) => return (StatusCode::BAD_REQUEST, "Failed to read request body").into_response(),
        };
        req = axum::http::Request::from_parts(parts, Body::from(bytes.clone()));
        req.extensions_mut().insert(bytes.clone());


        if let Ok(body_str) = std::str::from_utf8(&bytes) {
            if request_token.is_none() {
                request_token = extract_csrf_from_form(body_str);
            }
        }

        // Réinjection dans la requête ET dans les extensions

        // 3. Vérification
        match (session_token, request_token) {
            (Some(st), Some(rt)) if constant_time_compare(&st, &rt) => {
                let session = req.extensions().get::<Session>().unwrap();
                let session_id = session.id().map(|id| id.to_string()).unwrap_or_else(|| "no-session-id".to_string());
                let new_token = generate_csrf_token(&config.server.secret_key, &session_id);
                let _ = session.insert(CSRF_TOKEN_KEY, new_token.clone()).await;
                new_token
            }
            (_, _) => {
                let mut session = req.extensions().get::<Session>().cloned().expect("Session missing");
                let _ = session.insert_message(FlashMessage::error(
                    "Erreur de sécurité : Token CSRF invalide ou manquant."
                )).await;

                // Si c'est une requête AJAX/JSON, on renvoie un code d'erreur au lieu d'une redirection
                if headers.get("Accept").and_then(|h| h.to_str().ok()).unwrap_or("").contains("application/json") {
                    return (StatusCode::BAD_REQUEST, "Invalid CSRF Token").into_response();
                }

                return Redirect::to(&uri_path).into_response();
                        }
                    }
        } else {
        // === GET ===
        let session = match req.extensions().get::<Session>() {
            Some(s) => s,
            None => return (StatusCode::INTERNAL_SERVER_ERROR, "Session middleware missing").into_response(),
        };

        let existing = session.get::<String>(CSRF_TOKEN_KEY).await.ok().flatten();

        if let Some(t) = existing {
            t
        } else {
            let session_id = session.id().map(|id| id.to_string()).unwrap_or_else(|| "no-session-id".to_string());
            let new_token = generate_csrf_token(&config.server.secret_key, &session_id);
            let _ = session.insert(CSRF_TOKEN_KEY, new_token.clone()).await;
            new_token
        }
    };

    req.extensions_mut().insert(CsrfToken(token_to_inject.clone()));
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