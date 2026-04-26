//! Tests de sécurité — collisions de session, isolation, nettoyage au login.
//!
//! Ces tests vérifient les propriétés de sécurité critiques :
//! - Un login avec un user différent nettoie la session précédente
//! - Après logout, les données de session sont inaccessibles
//! - Le cache de permissions est bien évincé au logout
//! - Deux users distincts ne partagent pas de données de session

use axum::{Router, response::IntoResponse, routing::get};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

use runique::admin::permissions::{Groupe, Permission};
use runique::auth::guard::{cache_permissions, get_permissions};
use runique::auth::session::{get_user_id, get_username, is_authenticated, login, logout};

use crate::helpers::{
    assert::{assert_body_str, assert_status},
    request,
};

// ── Helper ────────────────────────────────────────────────────────────────────

fn build_app(handler: axum::routing::MethodRouter) -> Router {
    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store).with_secure(false);
    Router::new()
        .route("/test", get(handler))
        .layer(session_layer)
}

fn make_groupe(resource: &str) -> Groupe {
    Groupe {
        id: 1,
        nom: "test".to_string(),
        permissions: vec![Permission {
            resource_key: resource.to_string(),
            can_create: true,
            can_read: true,
            can_update: true,
            can_delete: true,
            can_update_own: false,
            can_delete_own: false,
        }],
    }
}

// ═══════════════════════════════════════════════════════════════
// Collision de session — login user B sur session user A
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_login_user_different_nettoie_session_precedente() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();

        // User A se connecte
        login(&session, &db, 1, "setsuna", true, false, None, false)
            .await
            .unwrap();
        let id_apres_login_a = get_user_id(&session).await;
        assert_eq!(id_apres_login_a, Some(1));

        // User B se connecte sur la même session (collision)
        login(&session, &db, 2, "itsuki", true, true, None, false)
            .await
            .unwrap();
        let id_apres_login_b = get_user_id(&session).await;
        let username_apres_login_b = get_username(&session).await;

        // La session doit appartenir à B, pas à A
        assert_eq!(id_apres_login_b, Some(2));
        assert_eq!(username_apres_login_b.as_deref(), Some("itsuki"));

        "ok"
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "ok").await;
}

#[tokio::test]
async fn test_login_meme_user_ne_reinitialise_pas_session() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();

        login(&session, &db, 1, "alice", true, false, None, false)
            .await
            .unwrap();
        // Re-login du même user (refresh de session)
        login(&session, &db, 1, "alice", true, false, None, false)
            .await
            .unwrap();

        let id = get_user_id(&session).await;
        assert_eq!(id, Some(1));
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
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();

        login(&session, &db, 1, "alice", true, false, None, false)
            .await
            .unwrap();
        assert!(is_authenticated(&session).await);

        logout(&session, None).await.unwrap();
        assert!(!is_authenticated(&session).await);
        assert!(get_user_id(&session).await.is_none());
        assert!(get_username(&session).await.is_none());

        "ok"
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "ok").await;
}

#[tokio::test]
async fn test_logout_evicte_cache_permissions() {
    let user_id: runique::utils::pk::Pk = 20_001;

    // Pré-charge le cache
    cache_permissions(user_id, vec![make_groupe("articles")]);
    assert!(get_permissions(user_id).is_some());

    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 20_001, "bob", true, false, None, false)
            .await
            .unwrap();
        logout(&session, None).await.unwrap();
        "ok"
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "ok").await;

    // Cache doit être évincé après logout
    assert!(get_permissions(user_id).is_none());
}

// ═══════════════════════════════════════════════════════════════
// Isolation — deux clients distincts
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_deux_sessions_independantes() {
    // Session A : user 1
    async fn handler_a(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 1, "alice", true, false, None, false)
            .await
            .unwrap();
        get_username(&session).await.unwrap_or_default()
    }

    // Session B : user 2 (router séparé = session store séparé)
    async fn handler_b(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 2, "bob", false, false, None, false)
            .await
            .unwrap();
        get_username(&session).await.unwrap_or_default()
    }

    let res_a = request::get(build_app(get(handler_a)), "/test").await;
    let res_b = request::get(build_app(get(handler_b)), "/test").await;

    assert_body_str(res_a, "alice").await;
    assert_body_str(res_b, "bob").await;
}

// ═══════════════════════════════════════════════════════════════
// Cache permissions — collision entre users
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_cache_permissions_isole_par_user_id() {
    let user_a: runique::utils::pk::Pk = 20_002;
    let user_b: runique::utils::pk::Pk = 20_003;

    cache_permissions(user_a, vec![make_groupe("articles")]);
    cache_permissions(user_b, vec![make_groupe("users")]);

    let perms_a = get_permissions(user_a).unwrap();
    let perms_b = get_permissions(user_b).unwrap();

    // A ne voit pas les permissions de B et vice versa
    assert_eq!(perms_a.groupes[0].permissions[0].resource_key, "articles");
    assert_eq!(perms_b.groupes[0].permissions[0].resource_key, "users");
    assert_ne!(
        perms_a.groupes[0].permissions[0].resource_key,
        perms_b.groupes[0].permissions[0].resource_key
    );
}

#[tokio::test]
async fn test_login_collision_nettoie_cache_ancien_user() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();

        // User A login — cache chargé
        login(&session, &db, 20_004, "carol", true, false, None, false)
            .await
            .unwrap();

        // Injecte manuellement des permissions pour A
        cache_permissions(20_004, vec![make_groupe("articles")]);
        assert!(get_permissions(20_004).is_some());

        // User B prend la session (collision)
        login(&session, &db, 20_005, "dave", true, false, None, false)
            .await
            .unwrap();

        // Le cache de A doit être évincé (logout interne)
        // B n'a pas de permissions en DB (sqlite memory vide) → cache vide
        let id = get_user_id(&session).await;
        assert_eq!(id, Some(20_005));

        "ok"
    }

    let res = request::get(build_app(get(handler)), "/test").await;
    assert_status(&res, 200);
    assert_body_str(res, "ok").await;

    // Cache de A évincé après la collision
    assert!(get_permissions(20_004).is_none());
}
