//! Tests — Session Auth (login, logout, is_authenticated)
//!
//! Ces tests utilisent un router Axum minimal avec MemoryStore pour créer
//! de vraies sessions sans démarrer de serveur persistant.

use axum::{Router, response::IntoResponse, routing::get};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

use runique::auth::session::{is_authenticated, login, logout, protect_session, unprotect_session};

use crate::helpers::{
    assert::{assert_body_str, assert_status},
    pk::pk,
    request,
    user::{session_user_id, test_user},
};

// ── Helper local ──────────────────────────────────────────────────────────────

fn build_app(handler: axum::routing::MethodRouter) -> Router {
    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store).with_secure(false);
    Router::new()
        .route("/test", get(handler))
        .layer(session_layer)
}

// ── is_authenticated ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_is_authenticated_when_no_user_in_session() {
    async fn handler(session: Session) -> impl IntoResponse {
        if is_authenticated(&session).await {
            "authenticated"
        } else {
            "anonymous"
        }
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "anonymous").await;
}

#[tokio::test]
async fn test_is_authenticated_after_login() {
    async fn handler(session: Session) -> impl IntoResponse {
        login(
            &session,
            &test_user(pk(1), "alice", false, false),
            None,
            false,
        )
        .await
        .unwrap();
        if is_authenticated(&session).await {
            "authenticated"
        } else {
            "anonymous"
        }
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "authenticated").await;
}

// ── login ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_login_stores_the_id_and_no_name() {
    async fn handler(session: Session) -> impl IntoResponse {
        login(
            &session,
            &test_user(pk(42), "bob", false, false),
            None,
            false,
        )
        .await
        .unwrap();
        let id = session_user_id(&session).await.unwrap_or_default();
        // The account is read from the database: no name copied into the session.
        let name = session.get::<String>("username").await.ok().flatten();
        format!("{}/{:?}", id, name)
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, &format!("{}/None", pk(42))).await;
}

// ── logout ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_is_not_authenticated_after_logout() {
    async fn handler(session: Session) -> impl IntoResponse {
        login(
            &session,
            &test_user(pk(1), "alice", false, false),
            None,
            false,
        )
        .await
        .unwrap();
        logout(&session, None).await.unwrap();
        if is_authenticated(&session).await {
            "authenticated"
        } else {
            "anonymous"
        }
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "anonymous").await;
}

// ── session id ────────────────────────────────────────────────

#[tokio::test]
async fn test_no_user_id_when_not_logged_in() {
    async fn handler(session: Session) -> impl IntoResponse {
        if session_user_id(&session).await.is_some() {
            "some"
        } else {
            "none"
        }
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "none").await;
}

// ── protect_session / unprotect_session ───────────────────────────────────────

#[tokio::test]
async fn test_protect_session_inserts_key() {
    use runique::utils::constante::session_key::session::SESSION_ACTIVE_KEY;
    async fn handler(session: Session) -> impl IntoResponse {
        protect_session(&session, 3600).await.unwrap();
        let key = session.get::<i64>(SESSION_ACTIVE_KEY).await.ok().flatten();
        if key.is_some() {
            "protected"
        } else {
            "missing"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "protected").await;
}

#[tokio::test]
async fn test_unprotect_session_removes_key() {
    use runique::utils::constante::session_key::session::SESSION_ACTIVE_KEY;
    async fn handler(session: Session) -> impl IntoResponse {
        protect_session(&session, 3600).await.unwrap();
        unprotect_session(&session).await.unwrap();
        let key = session.get::<i64>(SESSION_ACTIVE_KEY).await.ok().flatten();
        if key.is_none() {
            "removed"
        } else {
            "still_present"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "removed").await;
}
