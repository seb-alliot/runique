//! The session's lifetime, upgraded to the authenticated one once the user
//! is signed in (slot 55, right after the session layer).
use tower_sessions::{Expiry, cookie::time::Duration};

/// Upgrades session TTL to the authenticated duration when the user is logged in.
pub(crate) async fn session_ttl_upgrade(
    axum::extract::State(duration): axum::extract::State<Duration>,
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if let Some(session) = req.extensions().get::<tower_sessions::Session>()
        && crate::auth::session::is_authenticated(session).await
    {
        session.set_expiry(Some(Expiry::OnInactivity(duration)));
    }
    next.run(req).await
}
