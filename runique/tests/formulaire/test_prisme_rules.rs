// Tests pour GuardRules et evaluate_rules (module prisme/rules), évalués
// contre le `CurrentUser` relu en base.

use crate::helpers::pk::pk;
use axum::http::StatusCode;
use runique::auth::session::CurrentUser;
use runique::forms::prisme::rules::{GuardRules, evaluate_rules};

fn user(is_staff: bool, is_superuser: bool) -> CurrentUser {
    CurrentUser {
        id: pk(1),
        username: "alice".into(),
        is_staff,
        is_superuser,
        groupes: vec![],
    }
}

fn groupes(noms: &[&str]) -> Vec<String> {
    noms.iter().map(|n| n.to_string()).collect()
}

fn status(rules: &GuardRules, user: Option<&CurrentUser>, groupes: &[String]) -> StatusCode {
    match evaluate_rules(rules, user, groupes) {
        Ok(()) => StatusCode::OK,
        Err(resp) => resp.status(),
    }
}

// ═══════════════════════════════════════════════════════════════
// Constructeurs — toute règle exige un compte connecté
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_every_rule_requires_a_signed_in_account() {
    for rules in [
        GuardRules::login_required(),
        GuardRules::staff(),
        GuardRules::superuser(),
        GuardRules::roles(["editeur"]),
    ] {
        assert!(rules.login_required, "{rules:?}");
        assert_eq!(status(&rules, None, &[]), StatusCode::UNAUTHORIZED);
    }
}

#[test]
fn test_roles_keeps_every_group_named() {
    let rules = GuardRules::roles(["editeur", "moderateur"]);
    assert_eq!(rules.roles, groupes(&["editeur", "moderateur"]));
}

// ═══════════════════════════════════════════════════════════════
// evaluate_rules
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_no_rule_lets_anyone_through() {
    assert_eq!(status(&GuardRules::default(), None, &[]), StatusCode::OK);
}

#[test]
fn test_login_required_lets_any_account_through() {
    let rules = GuardRules::login_required();
    assert_eq!(
        status(&rules, Some(&user(false, false)), &[]),
        StatusCode::OK
    );
}

#[test]
fn test_staff_refuses_a_regular_account() {
    let rules = GuardRules::staff();
    assert_eq!(
        status(&rules, Some(&user(false, false)), &[]),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status(&rules, Some(&user(true, false)), &[]),
        StatusCode::OK
    );
    assert_eq!(
        status(&rules, Some(&user(false, true)), &[]),
        StatusCode::OK
    );
}

#[test]
fn test_superuser_refuses_staff() {
    let rules = GuardRules::superuser();
    assert_eq!(
        status(&rules, Some(&user(true, false)), &[]),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status(&rules, Some(&user(false, true)), &[]),
        StatusCode::OK
    );
}

#[test]
fn test_roles_needs_one_of_the_groups() {
    let rules = GuardRules::roles(["editeur", "moderateur"]);
    let alice = user(true, false);
    assert_eq!(
        status(&rules, Some(&alice), &groupes(&["moderateur"])),
        StatusCode::OK
    );
    assert_eq!(
        status(&rules, Some(&alice), &groupes(&["lecteur"])),
        StatusCode::FORBIDDEN
    );
    assert_eq!(status(&rules, Some(&alice), &[]), StatusCode::FORBIDDEN);
}

#[test]
fn test_roles_lets_a_superuser_through_without_groups() {
    let rules = GuardRules::roles(["editeur"]);
    assert_eq!(
        status(&rules, Some(&user(false, true)), &[]),
        StatusCode::OK
    );
}
