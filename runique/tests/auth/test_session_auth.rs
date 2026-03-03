//! Tests — Session Auth (login, login_staff, logout, is_authenticated)
//! Bug fix vérifié : logout() ne vidait pas toutes les clés de session
//!
//! Ces tests utilisent un router Axum minimal avec MemoryStore pour créer
//! de vraies sessions sans démarrer de serveur.

use axum::{response::IntoResponse, routing::get, Router};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

use runique::middleware::auth::{
    get_user_id, get_username, is_authenticated, login, login_staff, logout,
};
use runique::utils::constante::{
    SESSION_USER_IS_STAFF_KEY, SESSION_USER_IS_SUPERUSER_KEY, SESSION_USER_ROLES_KEY,
};

use crate::helpers::{
    assert::{assert_body_str, assert_status},
    request,
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
        login(&session, 1, "alice").await.unwrap();
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
async fn test_login_sets_id_and_username() {
    async fn handler(session: Session) -> impl IntoResponse {
        login(&session, 42, "bob").await.unwrap();
        let id = get_user_id(&session).await.unwrap_or(0);
        let username = get_username(&session).await.unwrap_or_default();
        format!("{}/{}", id, username)
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "42/bob").await;
}

// ── login_staff ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_login_staff_sets_all_fields() {
    async fn handler(session: Session) -> impl IntoResponse {
        login_staff(&session, 7, "admin", true, true, vec!["editor".to_string()])
            .await
            .unwrap();

        let id = get_user_id(&session).await.unwrap_or(0);
        let username = get_username(&session).await.unwrap_or_default();
        let is_staff = session
            .get::<bool>(SESSION_USER_IS_STAFF_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or(false);
        let is_su = session
            .get::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or(false);
        let roles = session
            .get::<Vec<String>>(SESSION_USER_ROLES_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or_default();

        format!(
            "{}/{}/{}/{}/{}",
            id,
            username,
            is_staff,
            is_su,
            roles.join(",")
        )
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "7/admin/true/true/editor").await;
}

// ── logout ────────────────────────────────────────────────────────────────────
// Bug fix : logout() ne vidait pas les clés de session

#[tokio::test]
async fn test_logout_clears_session_keys() {
    async fn handler(session: Session) -> impl IntoResponse {
        login_staff(
            &session,
            1,
            "alice",
            true,
            false,
            vec!["moderator".to_string()],
        )
        .await
        .unwrap();
        logout(&session).await.unwrap();

        let all_cleared = get_user_id(&session).await.is_none()
            && get_username(&session).await.is_none()
            && session
                .get::<bool>(SESSION_USER_IS_STAFF_KEY)
                .await
                .ok()
                .flatten()
                .is_none()
            && session
                .get::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
                .await
                .ok()
                .flatten()
                .is_none()
            && session
                .get::<Vec<String>>(SESSION_USER_ROLES_KEY)
                .await
                .ok()
                .flatten()
                .is_none();

        if all_cleared {
            "cleared"
        } else {
            "not_cleared"
        }
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "cleared").await;
}

#[tokio::test]
async fn test_is_not_authenticated_after_logout() {
    async fn handler(session: Session) -> impl IntoResponse {
        login(&session, 1, "alice").await.unwrap();
        logout(&session).await.unwrap();
        if is_authenticated(&session).await {
            "authenticated"
        } else {
            "anonymous"
        }
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "anonymous").await;
}

// ── get_user_id / get_username ────────────────────────────────────────────────

#[tokio::test]
async fn test_get_user_id_returns_none_when_not_logged_in() {
    async fn handler(session: Session) -> impl IntoResponse {
        if get_user_id(&session).await.is_some() {
            "some"
        } else {
            "none"
        }
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "none").await;
}

#[tokio::test]
async fn test_get_username_after_login() {
    async fn handler(session: Session) -> impl IntoResponse {
        login(&session, 1, "charlie").await.unwrap();
        get_username(&session).await.unwrap_or_default()
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "charlie").await;
}
