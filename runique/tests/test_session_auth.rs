//! Tests — Session Auth (login_user, logout, is_authenticated)
//! Bug fix vérifié : logout() ne vidait pas toutes les clés de session
//!
//! Ces tests utilisent un router Axum minimal avec MemoryStore pour créer
//! de vraies sessions sans démarrer de serveur.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use tower::ServiceExt;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

use runique::middleware::auth::{
    get_user_id, get_username, is_authenticated, login_user, login_user_full, logout,
};
use runique::utils::constante::{
    SESSION_USER_IS_STAFF_KEY, SESSION_USER_IS_SUPERUSER_KEY, SESSION_USER_ROLES_KEY,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_app(handler: axum::routing::MethodRouter) -> Router {
    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store).with_secure(false);
    Router::new()
        .route("/test", get(handler))
        .layer(session_layer)
}

async fn get_request(app: Router) -> axum::response::Response {
    app.oneshot(Request::builder().uri("/test").body(Body::empty()).unwrap())
        .await
        .unwrap()
}

// ── is_authenticated ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_is_authenticated_when_no_user_in_session() {
    async fn handler(session: Session) -> impl IntoResponse {
        let auth = is_authenticated(&session).await;
        if auth {
            "authenticated"
        } else {
            "anonymous"
        }
    }

    let app = build_app(get(handler));
    let res = get_request(app).await;
    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"anonymous");
}

#[tokio::test]
async fn test_is_authenticated_after_login() {
    async fn handler(session: Session) -> impl IntoResponse {
        login_user(&session, 1, "alice").await.unwrap();
        let auth = is_authenticated(&session).await;
        if auth {
            "authenticated"
        } else {
            "anonymous"
        }
    }

    let app = build_app(get(handler));
    let res = get_request(app).await;
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"authenticated");
}

// ── login_user ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_login_user_sets_id_and_username() {
    async fn handler(session: Session) -> impl IntoResponse {
        login_user(&session, 42, "bob").await.unwrap();
        let id = get_user_id(&session).await.unwrap_or(0);
        let username = get_username(&session).await.unwrap_or_default();
        format!("{}/{}", id, username)
    }

    let app = build_app(get(handler));
    let res = get_request(app).await;
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"42/bob");
}

// ── login_user_full ───────────────────────────────────────────────────────────

#[tokio::test]
async fn test_login_user_full_sets_all_fields() {
    async fn handler(session: Session) -> impl IntoResponse {
        login_user_full(&session, 7, "admin", true, true, vec!["editor".to_string()])
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

    let app = build_app(get(handler));
    let res = get_request(app).await;
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"7/admin/true/true/editor");
}

// ── logout ────────────────────────────────────────────────────────────────────
// Bug fix : logout() ne vidait pas les clés de session

#[tokio::test]
async fn test_logout_clears_session_keys() {
    async fn handler(session: Session) -> impl IntoResponse {
        // 1. Login complet
        login_user_full(
            &session,
            1,
            "alice",
            true,
            false,
            vec!["moderator".to_string()],
        )
        .await
        .unwrap();

        // 2. Logout
        logout(&session).await.unwrap();

        // 3. Vérifier que toutes les clés sont absentes
        let user_id = get_user_id(&session).await;
        let username = get_username(&session).await;
        let is_staff = session
            .get::<bool>(SESSION_USER_IS_STAFF_KEY)
            .await
            .ok()
            .flatten();
        let is_su = session
            .get::<bool>(SESSION_USER_IS_SUPERUSER_KEY)
            .await
            .ok()
            .flatten();
        let roles = session
            .get::<Vec<String>>(SESSION_USER_ROLES_KEY)
            .await
            .ok()
            .flatten();

        let all_cleared = user_id.is_none()
            && username.is_none()
            && is_staff.is_none()
            && is_su.is_none()
            && roles.is_none();

        if all_cleared {
            "cleared"
        } else {
            "not_cleared"
        }
    }

    let app = build_app(get(handler));
    let res = get_request(app).await;
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"cleared");
}

#[tokio::test]
async fn test_is_not_authenticated_after_logout() {
    async fn handler(session: Session) -> impl IntoResponse {
        login_user(&session, 1, "alice").await.unwrap();
        logout(&session).await.unwrap();
        let auth = is_authenticated(&session).await;
        if auth {
            "authenticated"
        } else {
            "anonymous"
        }
    }

    let app = build_app(get(handler));
    let res = get_request(app).await;
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"anonymous");
}

// ── get_user_id / get_username ────────────────────────────────────────────────

#[tokio::test]
async fn test_get_user_id_returns_none_when_not_logged_in() {
    async fn handler(session: Session) -> impl IntoResponse {
        let id = get_user_id(&session).await;
        if id.is_some() {
            "some"
        } else {
            "none"
        }
    }

    let app = build_app(get(handler));
    let res = get_request(app).await;
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"none");
}

#[tokio::test]
async fn test_get_username_after_login() {
    async fn handler(session: Session) -> impl IntoResponse {
        login_user(&session, 1, "charlie").await.unwrap();
        get_username(&session).await.unwrap_or_default()
    }

    let app = build_app(get(handler));
    let res = get_request(app).await;
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"charlie");
}
