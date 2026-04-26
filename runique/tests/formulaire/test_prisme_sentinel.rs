//! Tests — forms/prisme/sentinel.rs + rules.rs
//! Couvre : sentinel, GuardRules, GuardContext, evaluate_rules

use axum::{body::Body, http::Request};
use runique::config::app::RuniqueConfig;
use runique::forms::prisme::rules::{GuardContext, GuardRules};
use runique::forms::prisme::sentinel::sentinel;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn empty_request() -> Request<Body> {
    Request::builder().body(Body::empty()).unwrap()
}

fn config() -> RuniqueConfig {
    RuniqueConfig::default()
}

fn ctx_authenticated(roles: Vec<&str>) -> GuardContext {
    GuardContext {
        user_id: Some("1".to_string()),
        roles: roles.into_iter().map(|s| s.to_string()).collect(),
    }
}

fn ctx_anonymous() -> GuardContext {
    GuardContext::default()
}

// ═══════════════════════════════════════════════════════════════
// Sans règles injectées
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sentinel_sans_regles_retourne_ok() {
    let req = empty_request();
    let result = sentinel(&req, &config());
    assert!(
        result.is_ok(),
        "Sans GuardRules, sentinel doit retourner Ok"
    );
}

// ═══════════════════════════════════════════════════════════════
// login_required — sans contexte utilisateur
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sentinel_login_required_sans_contexte_retourne_err() {
    let mut req = empty_request();
    req.extensions_mut().insert(GuardRules::login_required());
    let result = sentinel(&req, &config());
    assert!(result.is_err(), "login_required sans contexte = Err");
}

#[test]
fn test_sentinel_login_required_avec_anonyme_retourne_err() {
    let mut req = empty_request();
    req.extensions_mut().insert(GuardRules::login_required());
    req.extensions_mut().insert(ctx_anonymous());
    let result = sentinel(&req, &config());
    assert!(result.is_err(), "login_required avec anonyme = Err");
}

#[test]
fn test_sentinel_login_required_avec_connecte_retourne_ok() {
    let mut req = empty_request();
    req.extensions_mut().insert(GuardRules::login_required());
    req.extensions_mut().insert(ctx_authenticated(vec![]));
    let result = sentinel(&req, &config());
    assert!(
        result.is_ok(),
        "login_required avec utilisateur connecté = Ok"
    );
}

// ═══════════════════════════════════════════════════════════════
// role requis
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sentinel_role_sans_role_retourne_err() {
    let mut req = empty_request();
    req.extensions_mut().insert(GuardRules::role("admin"));
    req.extensions_mut()
        .insert(ctx_authenticated(vec!["editor"]));
    let result = sentinel(&req, &config());
    assert!(result.is_err(), "Rôle non présent = Err");
}

#[test]
fn test_sentinel_role_avec_bon_role_retourne_ok() {
    let mut req = empty_request();
    req.extensions_mut().insert(GuardRules::role("admin"));
    req.extensions_mut()
        .insert(ctx_authenticated(vec!["admin"]));
    let result = sentinel(&req, &config());
    assert!(result.is_ok(), "Rôle présent = Ok");
}

#[test]
fn test_sentinel_role_parmi_plusieurs() {
    let mut req = empty_request();
    req.extensions_mut()
        .insert(GuardRules::roles(["admin", "moderator"]));
    req.extensions_mut()
        .insert(ctx_authenticated(vec!["moderator"]));
    let result = sentinel(&req, &config());
    assert!(result.is_ok(), "Un rôle valide parmi plusieurs = Ok");
}

// ═══════════════════════════════════════════════════════════════
// login + role
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_sentinel_login_and_role_ok() {
    let mut req = empty_request();
    req.extensions_mut()
        .insert(GuardRules::login_and_role("editor"));
    req.extensions_mut()
        .insert(ctx_authenticated(vec!["editor"]));
    let result = sentinel(&req, &config());
    assert!(result.is_ok(), "login + bon rôle = Ok");
}

#[test]
fn test_sentinel_login_and_role_mauvais_role() {
    let mut req = empty_request();
    req.extensions_mut()
        .insert(GuardRules::login_and_role("admin"));
    req.extensions_mut()
        .insert(ctx_authenticated(vec!["editor"]));
    let result = sentinel(&req, &config());
    assert!(result.is_err(), "login + mauvais rôle = Err");
}
