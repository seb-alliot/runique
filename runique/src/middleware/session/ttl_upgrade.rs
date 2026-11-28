//! The session's lifetime, upgraded to the authenticated one once the user
//! is signed in (slot 55, right after the session layer).
use crate::auth::session::is_authenticated;
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use tower_sessions::{Expiry, Session, cookie::time::Duration};

/// Upgrades session TTL to the authenticated duration when the user is logged in.
pub(crate) async fn session_ttl_upgrade(
    State(duration): State<Duration>,
    req: Request<Body>,
    next: Next,
) -> Response {
    if let Some(session) = req.extensions().get::<Session>()
        && is_authenticated(session).await
    {
        session.set_expiry(Some(Expiry::OnInactivity(duration)));
    }
    next.run(req).await
}
