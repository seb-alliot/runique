//! Tests — auth/session.rs : has_permission()
//!
//! Chaque test utilise un user_id distinct pour éviter les interférences
//! avec le cache global PERMISSIONS_CACHE (LazyLock).

use axum::{Router, response::IntoResponse, routing::get};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

use runique::admin::{Groupe, permissions::Permission};
use runique::auth::{
    guard::{cache_permissions, clear_cache, evict_permissions, get_permissions},
    session::{has_permission, login},
};

use crate::helpers::{assert::assert_body_str, request};

// ── Helper ─────────────────────────────────────────────────────────────────────

fn build_app(handler: axum::routing::MethodRouter) -> Router {
    let store = MemoryStore::default();
    let layer = SessionManagerLayer::new(store).with_secure(false);
    Router::new().route("/test", get(handler)).layer(layer)
}

fn perm(resource: &str, create: bool, read: bool, update: bool, delete: bool) -> Permission {
    Permission {
        resource_key: resource.to_string(),
        can_create: create,
        can_read: read,
        can_update: update,
        can_delete: delete,
        can_update_own: false,
        can_delete_own: false,
    }
}

fn groupe_with_perm(permission: Permission) -> Groupe {
    Groupe {
        id: 1,
        nom: "test_group".to_string(),
        permissions: vec![permission],
    }
}

// ═══════════════════════════════════════════════════════════════
// Superuser bypass
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_has_permission_superuser_bypass() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        // user_id 2001 : pas de cache → mais superuser contourne tout
        login(&session, &db, 2001, "su", false, true, None, false)
            .await
            .unwrap();
        if has_permission(&session, "anything.delete").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "ok").await;
}

// ═══════════════════════════════════════════════════════════════
// Pas de user_id en session
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_has_permission_no_user_id() {
    async fn handler(session: Session) -> impl IntoResponse {
        if has_permission(&session, "posts.read").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "denied").await;
}

// ═══════════════════════════════════════════════════════════════
// Pas de cache pour cet user_id
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_has_permission_no_cache() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        // user_id 2002 : pas de cache_permissions() appelé
        login(&session, &db, 2002, "nocache", false, false, None, false)
            .await
            .unwrap();
        // Vider le cache mis par login()
        runique::auth::guard::evict_permissions(2002);
        if has_permission(&session, "posts.read").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "denied").await;
}

// ═══════════════════════════════════════════════════════════════
// Permission "any" — user authentifié avec cache
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_has_permission_any_authenticated() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 2003, "anyuser", false, false, None, false)
            .await
            .unwrap();
        cache_permissions(2003, vec![]);
        if has_permission(&session, "any").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "ok").await;
}

// ═══════════════════════════════════════════════════════════════
// Permission par action
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_has_permission_read_true() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 2004, "reader", false, false, None, false)
            .await
            .unwrap();
        let groupes = vec![groupe_with_perm(perm("posts", false, true, false, false))];
        cache_permissions(2004, groupes);
        if has_permission(&session, "posts.read").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "ok").await;
}

#[tokio::test]
async fn test_has_permission_create_false() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 2005, "reader2", false, false, None, false)
            .await
            .unwrap();
        // only read, no create
        let groupes = vec![groupe_with_perm(perm("posts", false, true, false, false))];
        cache_permissions(2005, groupes);
        if has_permission(&session, "posts.create").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "denied").await;
}

#[tokio::test]
async fn test_has_permission_update_true() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 2006, "editor", false, false, None, false)
            .await
            .unwrap();
        let groupes = vec![groupe_with_perm(perm("posts", false, false, true, false))];
        cache_permissions(2006, groupes);
        if has_permission(&session, "posts.update").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "ok").await;
}

#[tokio::test]
async fn test_has_permission_delete_true() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 2007, "deleter", false, false, None, false)
            .await
            .unwrap();
        let groupes = vec![groupe_with_perm(perm("posts", false, false, false, true))];
        cache_permissions(2007, groupes);
        if has_permission(&session, "posts.delete").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "ok").await;
}

#[tokio::test]
async fn test_has_permission_update_own_true() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 2008, "own_editor", false, false, None, false)
            .await
            .unwrap();
        let groupes = vec![Groupe {
            id: 1,
            nom: "g".to_string(),
            permissions: vec![Permission {
                resource_key: "posts".to_string(),
                can_create: false,
                can_read: false,
                can_update: false,
                can_delete: false,
                can_update_own: true,
                can_delete_own: false,
            }],
        }];
        cache_permissions(2008, groupes);
        if has_permission(&session, "posts.update_own").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "ok").await;
}

#[tokio::test]
async fn test_has_permission_resource_only_any_action() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 2009, "anyaction", false, false, None, false)
            .await
            .unwrap();
        // resource sans point → vérifie "any"
        let groupes = vec![groupe_with_perm(perm("posts", true, false, false, false))];
        cache_permissions(2009, groupes);
        if has_permission(&session, "posts").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "ok").await;
}

#[tokio::test]
async fn test_has_permission_unknown_action_false() {
    async fn handler(session: Session) -> impl IntoResponse {
        let db = sea_orm::Database::connect("sqlite::memory:").await.unwrap();
        login(&session, &db, 2010, "u", false, false, None, false)
            .await
            .unwrap();
        let groupes = vec![groupe_with_perm(perm("posts", true, true, true, true))];
        cache_permissions(2010, groupes);
        if has_permission(&session, "posts.fly").await {
            "ok"
        } else {
            "denied"
        }
    }
    let res = request::get(build_app(get(handler)), "/test").await;
    assert_body_str(res, "denied").await;
}

// ═══════════════════════════════════════════════════════════════
// cache_permissions / get_permissions / evict_permissions / clear_cache
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cache_permissions_and_get() {
    cache_permissions(3001, vec![]);
    let result = get_permissions(3001);
    assert!(result.is_some());
}

#[test]
fn test_get_permissions_absent_returns_none() {
    evict_permissions(3002);
    let result = get_permissions(3002);
    assert!(result.is_none());
}

#[test]
fn test_evict_permissions_removes_entry() {
    cache_permissions(3003, vec![]);
    assert!(get_permissions(3003).is_some());
    evict_permissions(3003);
    assert!(get_permissions(3003).is_none());
}

#[test]
fn test_evict_nonexistent_is_noop() {
    evict_permissions(3004);
}

#[test]
fn test_clear_cache_removes_entries() {
    cache_permissions(3005, vec![]);
    cache_permissions(3006, vec![]);
    clear_cache();
    assert!(get_permissions(3005).is_none());
    assert!(get_permissions(3006).is_none());
}
