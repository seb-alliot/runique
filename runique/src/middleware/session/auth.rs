//! The signed-in account, read again on every request (slot 57, after the
//! session layer): `CurrentUser` when it may still sign in, the session
//! closed otherwise.
use crate::{
    auth::{permissions::rank::Rang, user_trait::RuniqueUser},
    http::RequestExtensions,
    translation::t,
    utils::aliases::AEngine,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use tower_sessions::Session;
use tracing::Level;

/// Injects the signed-in user as the database knows them now.
///
/// The session only holds who signed in: the account is read on every request
/// of a signed-in user (one lookup by primary key), so deactivating it, or
/// increasing its `session_version`, applies to the next request — its session
/// is closed and `req.user` is `None`.
/// Groups and rights are not loaded here (`Request::load_user_rights`).
pub(crate) async fn auth_middleware(mut req: Request<Body>, next: Next) -> Response {
    use crate::{
        auth::{
            current_user::CurrentUser,
            session::{get_user_id, logout, session_is_current},
            user::BuiltinUserEntity,
        },
        log::TraceResult,
    };
    use axum::response::IntoResponse;

    let session = req.extensions().get::<Session>().cloned();
    let engine = req.extensions().get::<AEngine>().cloned();
    if let (Some(session), Some(engine)) = (session, engine)
        && let Some(id) = get_user_id(&session).await
    {
        let account = match BuiltinUserEntity::lookup_by_id(&engine.db, id).await {
            Ok(account) => account.filter(RuniqueUser::can_sign_in),
            // The database didn't answer: that says nothing about the account.
            // Its session is left as it is, and the request waits for better.
            Err(error) => {
                crate::runique_log!(
                    Level::WARN,
                    user_id = %id,
                    error = %error,
                    "{}", t(runique_i18n::keys::auth::log::ACCOUNT_NOT_READ)
                );
                return StatusCode::SERVICE_UNAVAILABLE.into_response();
            }
        };
        let account = match account {
            Some(user) if session_is_current(&session, &user).await => Some(user),
            _ => None,
        };
        match account {
            Some(user) => {
                let current_user = CurrentUser {
                    id,
                    rang: Rang::from(&user),
                    username: user.username,
                    groupes: Vec::new(),
                    droits: Vec::new(),
                };
                RequestExtensions::new()
                    .with_current_user(current_user)
                    .inject_request(&mut req);
            }
            None => {
                let db_store = engine.sessions.database.clone();
                // Logged, not fatal: the account is read again on every request and
                // refused here each time, so a row that survives grants nothing.
                logout(&session, db_store.as_deref()).await.trace_or(
                    crate::log::log_level!(session.store),
                    Level::WARN,
                    "close the session of a deleted or inactive account, or an outdated one",
                );
            }
        }
    }
    next.run(req).await
}
