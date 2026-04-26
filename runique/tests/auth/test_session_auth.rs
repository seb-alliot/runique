//! Tests — Session Auth (login, logout, is_authenticated)
//!
//! Ces tests utilisent un router Axum minimal avec MemoryStore pour créer
//! de vraies sessions sans démarrer de serveur persistant.

use axum::{Router, response::IntoResponse, routing::get};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

use runique::admin::Groupe;
use runique::auth::session::{
    get_user_id, get_username, is_admin_authenticated, is_authenticated, login, logout,
    protect_session, unprotect_session,
};
use runique::utils::constante::{
    admin_context::permission::GROUPES,
    session_key::session::{SESSION_USER_IS_STAFF_KEY, SESSION_USER_IS_SUPERUSER_KEY},
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
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 1, "alice", false, false, None, false)
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
async fn test_login_sets_id_and_username() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 42, "bob", false, false, None, false)
            .await
            .unwrap();
        let id = get_user_id(&session).await.unwrap_or(0);
        let username = get_username(&session).await.unwrap_or_default();
        format!("{}/{}", id, username)
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "42/bob").await;
}

// ── login — tous les champs ───────────────────────────────────────────────────

#[tokio::test]
async fn test_login_sets_all_fields() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 7, "admin", true, true, None, false)
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
        let groupes = session
            .get::<Vec<Groupe>>(GROUPES)
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
            groupes.len()
        )
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "7/admin/true/true/0").await;
}

// ── logout ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_logout_clears_session_keys() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 1, "alice", true, false, None, false)
            .await
            .unwrap();
        logout(&session, None).await.unwrap();

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
                .get::<Vec<Groupe>>(GROUPES)
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
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 1, "alice", false, false, None, false)
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
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 1, "charlie", false, false, None, false)
            .await
            .unwrap();
        get_username(&session).await.unwrap_or_default()
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "charlie").await;
}

// ── is_admin_authenticated ────────────────────────────────────────────────────

#[tokio::test]
async fn test_is_admin_authenticated_not_logged_in() {
    async fn handler(session: Session) -> impl IntoResponse {
        if is_admin_authenticated(&session).await {
            "admin"
        } else {
            "not_admin"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "not_admin").await;
}

#[tokio::test]
async fn test_is_admin_authenticated_plain_user() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 10, "user", false, false, None, false)
            .await
            .unwrap();
        if is_admin_authenticated(&session).await {
            "admin"
        } else {
            "not_admin"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "not_admin").await;
}

#[tokio::test]
async fn test_is_admin_authenticated_staff() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 11, "staff", true, false, None, false)
            .await
            .unwrap();
        if is_admin_authenticated(&session).await {
            "admin"
        } else {
            "not_admin"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "admin").await;
}

#[tokio::test]
async fn test_is_admin_authenticated_superuser() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 12, "su", false, true, None, false)
            .await
            .unwrap();
        if is_admin_authenticated(&session).await {
            "admin"
        } else {
            "not_admin"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "admin").await;
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
