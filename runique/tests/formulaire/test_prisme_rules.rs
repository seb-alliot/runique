// Tests pour GuardRules et evaluate_rules (module prisme/rules), évalués
// contre le `CurrentUser` relu en base.

use crate::helpers::pk::pk;
use axum::http::StatusCode;
use runique::auth::permissions::Groupe;
use runique::auth::rang::Rang;
use runique::auth::session::CurrentUser;
use runique::forms::prisme::rules::{GuardRules, evaluate_rules};

/// `staff_group`: member of one group that opens the admin.
fn user(rang: Rang, staff_group: bool) -> CurrentUser {
    let groupes = if staff_group {
        vec![Groupe {
            id: 1,
            nom: "equipe".into(),
            is_staff: true,
            permissions: vec![],
        }]
    } else {
        vec![]
    };
    CurrentUser {
        id: pk(1),
        username: "alice".into(),
        rang,
        groupes,
        droits: Vec::new(),
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
        status(&rules, Some(&user(Rang::Lambda, false)), &[]),
        StatusCode::OK
    );
}

#[test]
fn test_staff_refuses_a_regular_account() {
    let rules = GuardRules::staff();
    assert_eq!(
        status(&rules, Some(&user(Rang::Lambda, false)), &[]),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status(&rules, Some(&user(Rang::Lambda, true)), &[]),
        StatusCode::OK
    );
    assert_eq!(
        status(&rules, Some(&user(Rang::Superuser, false)), &[]),
        StatusCode::OK
    );
}

#[test]
fn test_staff_lets_any_rank_through_without_a_staff_group() {
    let rules = GuardRules::staff();
    for rang in [Rang::Admin, Rang::Dirigeant, Rang::Superuser] {
        assert_eq!(
            status(&rules, Some(&user(rang, false)), &[]),
            StatusCode::OK,
            "{rang:?}"
        );
    }
}

#[test]
fn test_staff_refuses_a_group_that_does_not_open_the_admin() {
    let mut alice = user(Rang::Lambda, true);
    alice.groupes[0].is_staff = false;
    assert_eq!(
        status(&GuardRules::staff(), Some(&alice), &[]),
        StatusCode::FORBIDDEN
    );
}

#[test]
fn test_superuser_refuses_every_lower_rank() {
    let rules = GuardRules::superuser();
    for rang in [Rang::Lambda, Rang::Admin, Rang::Dirigeant] {
        assert_eq!(
            status(&rules, Some(&user(rang, false)), &[]),
            StatusCode::FORBIDDEN,
            "{rang:?}"
        );
    }
}

#[test]
fn test_superuser_refuses_staff() {
    let rules = GuardRules::superuser();
    assert_eq!(
        status(&rules, Some(&user(Rang::Lambda, true)), &[]),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status(&rules, Some(&user(Rang::Superuser, false)), &[]),
        StatusCode::OK
    );
}

#[test]
fn test_roles_needs_one_of_the_groups() {
    let rules = GuardRules::roles(["editeur", "moderateur"]);
    let alice = user(Rang::Lambda, true);
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
        status(&rules, Some(&user(Rang::Superuser, false)), &[]),
        StatusCode::OK
    );
}
