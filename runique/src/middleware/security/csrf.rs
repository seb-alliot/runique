use crate::context::RequestExtensions;
use crate::middleware::auth::is_authenticated;
use crate::utils::aliases::{AEngine, JsonMap, TResult};
use crate::utils::constante::{CSRF_TOKEN_KEY, SESSION_USER_ID_KEY};
use crate::utils::csrf::{CsrfContext, CsrfToken};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
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
    let secret = &engine.config.server.secret_key;

    // Récupérer ou générer le token de session
    let session_token: CsrfToken = match session
        .get::<CsrfToken>(CSRF_TOKEN_KEY)
        .await
        .ok()
        .flatten()
    {
        Some(t) => {
            session.insert(CSRF_TOKEN_KEY, &t).await.expect("...");
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
            session.insert(CSRF_TOKEN_KEY, &token).await.expect("...");
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
                Some(token) if token.as_str() == session_token.as_str() => {
                    // OK, continue
                }
                _ => {
                    return (StatusCode::FORBIDDEN, "Invalid CSRF token").into_response();
                }
            }
        }
        // Sinon, on laisse Prisme valider le champ de formulaire
    }

    // Injection du token pour le frontend
    let masked = session_token.masked();
    let extensions = RequestExtensions::new().with_csrf_token(session_token.clone());
    extensions.inject_request(&mut req);

    let mut res = next.run(req).await;

    if let Ok(hv) = HeaderValue::from_str(masked.as_str()) {
        res.headers_mut().insert("X-CSRF-Token", hv);
    }

    res
}
