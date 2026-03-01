//! Tests — config/router.rs
//! Couvre : RuniqueRouter::new, default, add_route, nest

use axum::{routing::get, Router};
use runique::config::router::RuniqueRouter;
use runique::utils::aliases::AEngine;

// ═══════════════════════════════════════════════════════════════
// Constructeurs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_router_new_ne_panique_pas() {
    let _router = RuniqueRouter::new();
}

#[test]
fn test_runique_router_default_ne_panique_pas() {
    let _router = RuniqueRouter::default();
}

// ═══════════════════════════════════════════════════════════════
// add_route
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_add_route_simple() {
    let _router = RuniqueRouter::new().add_route("/test", get(|| async { "ok" }));
}

#[test]
fn test_add_route_multiple() {
    let _router = RuniqueRouter::new()
        .add_route("/a", get(|| async { "a" }))
        .add_route("/b", get(|| async { "b" }))
        .add_route("/c", get(|| async { "c" }));
}

#[test]
fn test_add_route_chainage_fluent() {
    let _router = RuniqueRouter::new()
        .add_route("/liste", get(|| async { "liste" }))
        .add_route("/detail", get(|| async { "detail" }));
}

// ═══════════════════════════════════════════════════════════════
// nest
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_nest_router() {
    let inner: Router<AEngine> = Router::new().route("/inner", get(|| async { "nested" }));
    let _router = RuniqueRouter::new().nest("/api", inner);
}

#[test]
fn test_nest_multiple_routers() {
    let api: Router<AEngine> = Router::new().route("/list", get(|| async { "list" }));
    let admin: Router<AEngine> = Router::new().route("/dashboard", get(|| async { "dashboard" }));
    let _router = RuniqueRouter::new().nest("/api", api).nest("/admin", admin);
}

// ═══════════════════════════════════════════════════════════════
// Combinaison add_route + nest
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_add_route_et_nest_combines() {
    let inner: Router<AEngine> = Router::new().route("/item", get(|| async { "item" }));
    let _router = RuniqueRouter::new()
        .add_route("/home", get(|| async { "home" }))
        .nest("/api", inner);
}
