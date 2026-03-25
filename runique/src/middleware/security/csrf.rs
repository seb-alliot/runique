use crate::context::RequestExtensions;
use crate::middleware::auth::is_authenticated;
use crate::utils::{
    aliases::{AEngine, JsonMap, TResult},
    constante::{CSRF_TOKEN_KEY, SESSION_USER_ID_KEY},
    csrf::{CsrfContext, CsrfToken},
};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use subtle::ConstantTimeEq;
use tera::{Function, Value};
use tower_sessions::Session;

pub struct CsrfTokenFunction;

impl Function for CsrfTokenFunction {
    fn is_safe(&self) -> bool {
        true
    }

    fn call(&self, args: &JsonMap) -> TResult {
        let token_str = args
            .get(CSRF_TOKEN_KEY)
            .and_then(|v| v.as_str())
            .unwrap_or("");

        Ok(Value::String(format!(
            r#"<input type="hidden" name="csrf_token" value="{}">"#,
            token_str
        )))
    }
}

pub async fn csrf_middleware(
    State(engine): State<AEngine>,
    session: Session,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // Strip csrf_token des query params GET pour éviter l'exposition du token dans les URLs
    if matches!(req.method(), &Method::GET | &Method::HEAD) {
        let uri = req.uri();
        if let Some(query) = uri.query() {
            if query.split('&').any(|p| p.starts_with("csrf_token=")) {
                let clean_query: String = query
                    .split('&')
                    .filter(|p| !p.starts_with("csrf_token="))
                    .collect::<Vec<_>>()
                    .join("&");

                let new_uri = if clean_query.is_empty() {
                    uri.path().to_string()
                } else {
                    format!("{}?{}", uri.path(), clean_query)
                };

                if let Ok(location) = HeaderValue::from_str(&new_uri) {
                    let mut res = (StatusCode::FOUND, "").into_response();
                    res.headers_mut()
                        .insert(axum::http::header::LOCATION, location);
                    return res;
                }
            }
        }
    }

    let secret = &engine.config.server.secret_key;

    // Récupérer ou générer le token de session
    let session_token: CsrfToken = match session
        .get::<CsrfToken>(CSRF_TOKEN_KEY)
        .await
        .ok()
        .flatten()
    {
        Some(t) => {
            if session.insert(CSRF_TOKEN_KEY, &t).await.is_err() {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Session write error").into_response();
            }
            t
        }
        None => {
            let token = if is_authenticated(&session).await {
                let user_id: i32 = session
                    .get::<i32>(SESSION_USER_ID_KEY)
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
            if session.insert(CSRF_TOKEN_KEY, &token).await.is_err() {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Session write error").into_response();
            }
            token
        }
    };

    // Vérification CSRF **UNIQUEMENT pour les requêtes AJAX avec header**
    let requires_csrf = matches!(
        req.method(),
        &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH
    );

    if requires_csrf {
        let has_header = req.headers().contains_key("X-CSRF-Token");

        // Si header présent, on valide (requête AJAX)
        if has_header {
            let header_token = req
                .headers()
                .get("X-CSRF-Token")
                .and_then(|h| h.to_str().ok())
                .and_then(|masked| CsrfToken::unmasked(masked).ok());

            match header_token {
                Some(token)
                    if token
                        .as_str()
                        .as_bytes()
                        // ct_eq : comparaison constant-time — évite qu'un attaquant
                        // devine le token octet par octet via le temps de réponse
                        .ct_eq(session_token.as_str().as_bytes())
                        .into() =>
                {
                    // OK, continue
                }
                _ => {
                    return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
                }
            }
        } else {
            // Pas de header CSRF : autorisé uniquement pour les soumissions de formulaires HTML
            // (urlencoded / multipart). Les requêtes JSON sans header sont bloquées.
            let ct = req
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let is_form = ct.starts_with("application/x-www-form-urlencoded")
                || ct.starts_with("multipart/form-data");
            if !is_form {
                return (StatusCode::FORBIDDEN, "CSRF token required").into_response();
            }
            // Sinon, on laisse Prisme valider le champ de formulaire
        }
    }

    // Injection du token pour le frontend
    let masked = session_token
        .masked()
        .unwrap_or_else(|_| session_token.clone());
    let extensions = RequestExtensions::new().with_csrf_token(session_token.clone());
    extensions.inject_request(&mut req);

    let mut res = next.run(req).await;

    if let Ok(hv) = HeaderValue::from_str(masked.as_str()) {
        res.headers_mut().insert("X-CSRF-Token", hv);
    }

    res
}
