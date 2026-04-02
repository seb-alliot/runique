//! Tests — CurrentUser
//! Couvre : has_droit, has_any_droit, can_access_admin, can_admin

use runique::admin::Droit;
use runique::middleware::auth::CurrentUser;

fn user(is_staff: bool, is_superuser: bool, droits: Vec<&str>) -> CurrentUser {
    CurrentUser {
        id: 1,
        username: "alice".to_string(),
        is_staff,
        is_superuser,
        droits: droits
            .iter()
            .enumerate()
            .map(|(i, n)| Droit {
                id: i as i32 + 1,
                nom: n.to_string(),
            })
            .collect(),
        groupes: vec![],
    }
}

// ── has_droit ─────────────────────────────────────────────────────────────────

#[test]
fn test_has_droit_matching() {
    let u = user(false, false, vec!["editor", "moderator"]);
    assert!(u.has_droit("editor"));
    assert!(u.has_droit("moderator"));
}

#[test]
fn test_has_droit_no_match() {
    let u = user(false, false, vec!["editor"]);
    assert!(!u.has_droit("admin"));
}

#[test]
fn test_has_droit_empty_droits() {
    let u = user(false, false, vec![]);
    assert!(!u.has_droit("editor"));
}

// ── has_any_droit ─────────────────────────────────────────────────────────────

#[test]
fn test_has_any_droit_one_matches() {
    let u = user(false, false, vec!["editor"]);
    assert!(u.has_any_droit(&["viewer", "editor"]));
}

#[test]
fn test_has_any_droit_none_match() {
    let u = user(false, false, vec!["editor"]);
    assert!(!u.has_any_droit(&["admin", "superuser"]));
}

#[test]
fn test_has_any_droit_empty_required() {
    let u = user(false, false, vec!["editor"]);
    assert!(!u.has_any_droit(&[]));
}

// ── can_access_admin ──────────────────────────────────────────────────────────

#[test]
fn test_can_access_admin_is_staff() {
    let u = user(true, false, vec![]);
    assert!(u.can_access_admin());
}

#[test]
fn test_can_access_admin_is_superuser() {
    let u = user(false, true, vec![]);
    assert!(u.can_access_admin());
}

#[test]
fn test_can_access_admin_neither() {
    let u = user(false, false, vec![]);
    assert!(!u.can_access_admin());
}

#[test]
fn test_can_access_admin_both() {
    let u = user(true, true, vec![]);
    assert!(u.can_access_admin());
}

// ── can_admin ─────────────────────────────────────────────────────────────────

#[test]
fn test_can_admin_superuser_bypasses_all() {
    let u = user(false, true, vec![]);
    assert!(u.can_admin(&["admin", "restricted"]));
}

#[test]
fn test_can_admin_has_required_droit() {
    let u = user(false, false, vec!["editor"]);
    assert!(u.can_admin(&["editor"]));
}

#[test]
fn test_can_admin_missing_required_droit() {
    let u = user(false, false, vec!["viewer"]);
    assert!(!u.can_admin(&["editor", "admin"]));
}

#[test]
fn test_can_admin_empty_droits_not_superuser() {
    let u = user(false, false, vec![]);
    assert!(!u.can_admin(&["admin"]));
}
