//! Tests de sécurité — collisions de session, isolation, nettoyage au login.
//!
//! Ces tests vérifient les propriétés de sécurité critiques :
//! - Un login avec un user différent nettoie la session précédente
//! - Après logout, les données de session sont inaccessibles
//! - Deux users distincts ne partagent pas de données de session

use axum::{Router, response::IntoResponse, routing::get};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

use crate::helpers::user::session_user_id;
use runique::auth::session::{is_authenticated, login, logout};

use crate::helpers::{
    assert::{assert_body_str, assert_status},
    pk::pk,
    request,
    user::test_user,
};

// ── Helper ────────────────────────────────────────────────────────────────────

fn build_app(handler: axum::routing::MethodRouter) -> Router {
    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store).with_secure(false);
    Router::new()
        .route("/test", get(handler))
        .layer(session_layer)
}

// ═══════════════════════════════════════════════════════════════
// Collision de session — login user B sur session user A
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_login_user_different_nettoie_session_precedente() {
    async fn handler(session: Session) -> impl IntoResponse {
        // User A se connecte
        login(
            &session,
            &test_user(pk(1), "setsuna", true, false),
            None,
            false,
        )
        .await
        .unwrap();
        let id_apres_login_a = session_user_id(&session).await;
        assert_eq!(id_apres_login_a, Some(pk(1)));

        // User B se connecte sur la même session (collision)
        login(
            &session,
            &test_user(pk(2), "itsuki", true, true),
            None,
            false,
        )
        .await
        .unwrap();
        let id_apres_login_b = session_user_id(&session).await;

        // La session doit appartenir à B, pas à A
        assert_eq!(id_apres_login_b, Some(pk(2)));

        "ok"
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "ok").await;
}

#[tokio::test]
async fn test_login_meme_user_ne_reinitialise_pas_session() {
    async fn handler(session: Session) -> impl IntoResponse {
        login(
            &session,
            &test_user(pk(1), "alice", true, false),
            None,
            false,
        )
        .await
        .unwrap();
        // Re-login du même user (refresh de session)
        login(
            &session,
            &test_user(pk(1), "alice", true, false),
            None,
            false,
        )
        .await
        .unwrap();

        let id = session_user_id(&session).await;
        assert_eq!(id, Some(pk(1)));
        "ok"
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "ok").await;
}

#[tokio::test]
async fn test_login_user_different_efface_les_donnees_de_a() {
    async fn handler(session: Session) -> impl IntoResponse {
        login(
            &session,
            &test_user(pk(1), "setsuna", true, false),
            None,
            false,
        )
        .await
        .unwrap();
        session.insert("panier", "article de A").await.unwrap();

        login(
            &session,
            &test_user(pk(2), "itsuki", true, false),
            None,
            false,
        )
        .await
        .unwrap();

        let panier: Option<String> = session.get("panier").await.unwrap();
        assert_eq!(panier, None, "les données de A ne doivent pas passer à B");
        "ok"
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "ok").await;
}

#[tokio::test]
async fn test_relogin_meme_user_garde_ses_donnees() {
    async fn handler(session: Session) -> impl IntoResponse {
        login(
            &session,
            &test_user(pk(1), "alice", true, false),
            None,
            false,
        )
        .await
        .unwrap();
        session.insert("panier", "article").await.unwrap();

        login(
            &session,
            &test_user(pk(1), "alice", true, false),
            None,
            false,
        )
        .await
        .unwrap();

        let panier: Option<String> = session.get("panier").await.unwrap();
        assert_eq!(panier.as_deref(), Some("article"));
        "ok"
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "ok").await;
}

// ═══════════════════════════════════════════════════════════════
// Logout — nettoyage complet
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_logout_vide_session_completement() {
    async fn handler(session: Session) -> impl IntoResponse {
        login(
            &session,
            &test_user(pk(1), "alice", true, false),
            None,
            false,
        )
        .await
        .unwrap();
        assert!(is_authenticated(&session).await);

        logout(&session, None).await.unwrap();
        assert!(!is_authenticated(&session).await);
        assert!(session_user_id(&session).await.is_none());

        "ok"
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "ok").await;
}

// ═══════════════════════════════════════════════════════════════
// Isolation — deux clients distincts
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_deux_sessions_independantes() {
    // Session A : user 1
    async fn handler_a(session: Session) -> impl IntoResponse {
        login(
            &session,
            &test_user(pk(1), "alice", true, false),
            None,
            false,
        )
        .await
        .unwrap();
        session_user_id(&session)
            .await
            .map(|id| id.to_string())
            .unwrap_or_default()
    }

    // Session B : user 2 (router séparé = session store séparé)
    async fn handler_b(session: Session) -> impl IntoResponse {
        login(
            &session,
            &test_user(pk(2), "bob", false, false),
            None,
            false,
        )
        .await
        .unwrap();
        session_user_id(&session)
            .await
            .map(|id| id.to_string())
            .unwrap_or_default()
    }

    let res_a = request::get(build_app(get(handler_a)), "/test").await;
    let res_b = request::get(build_app(get(handler_b)), "/test").await;

    assert_body_str(res_a, &pk(1).to_string()).await;
    assert_body_str(res_b, &pk(2).to_string()).await;
}

// ═══════════════════════════════════════════════════════════════
// Cache permissions — collision entre users
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_login_collision_bascule_sur_le_nouvel_user() {
    async fn handler(session: Session) -> impl IntoResponse {
        // User A login
        login(
            &session,
            &test_user(pk(20_004), "carol", true, false),
            None,
            false,
        )
        .await
        .unwrap();

        // User B prend la session (collision)
        login(
            &session,
            &test_user(pk(20_005), "dave", true, false),
            None,
            false,
        )
        .await
        .unwrap();

        let id = session_user_id(&session).await;
        assert_eq!(id, Some(pk(20_005)));

        "ok"
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "ok").await;
}
