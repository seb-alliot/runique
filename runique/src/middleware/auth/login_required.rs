//! Middleware `login_required` — redirige vers la page de login si l'utilisateur n'est pas authentifié.
use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use crate::middleware::auth::is_authenticated;

/// Middleware qui redirige vers `redirect_url` si l'utilisateur n'est pas authentifié.
pub async fn login_required_middleware(
    State(redirect_url): State<Arc<String>>,
    session: Session,
    req: Request<Body>,
    next: Next,
) -> Response {
    if is_authenticated(&session).await {
        next.run(req).await
    } else {
        Redirect::to(redirect_url.as_str()).into_response()
    }
}
