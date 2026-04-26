// Tests pour GuardContext, GuardRules et evaluate_rules (module prisme/rules)

use axum::http::StatusCode;
use runique::forms::prisme::rules::{GuardContext, GuardRules, evaluate_rules};

// ═══════════════════════════════════════════════════════════════
// GuardContext
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_guard_context_default_non_authentifie() {
    let ctx = GuardContext::default();
    assert!(!ctx.is_authenticated());
    assert!(ctx.roles.is_empty());
}

#[test]
fn test_guard_context_authentifie() {
    let ctx = GuardContext {
        user_id: Some("user-42".to_string()),
        roles: vec![],
    };
    assert!(ctx.is_authenticated());
}

#[test]
fn test_guard_context_non_authentifie() {
    let ctx = GuardContext {
        user_id: None,
        roles: vec!["admin".to_string()],
    };
    assert!(!ctx.is_authenticated());
}

#[test]
fn test_guard_context_has_role_present() {
    let ctx = GuardContext {
        user_id: Some("user".to_string()),
        roles: vec!["admin".to_string(), "moderateur".to_string()],
    };
    assert!(ctx.has_role("admin"));
    assert!(ctx.has_role("moderateur"));
}

#[test]
fn test_guard_context_has_role_absent() {
    let ctx = GuardContext {
        user_id: Some("user".to_string()),
        roles: vec!["editeur".to_string()],
    };
    assert!(!ctx.has_role("admin"));
}

#[test]
fn test_guard_context_has_role_liste_vide() {
    let ctx = GuardContext {
        user_id: Some("user".to_string()),
        roles: vec![],
    };
    assert!(!ctx.has_role("admin"));
}

// ═══════════════════════════════════════════════════════════════
// GuardRules — constructeurs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_guard_rules_default() {
    let rules = GuardRules::default();
    assert!(!rules.login_required);
    assert!(rules.roles.is_empty());
}

#[test]
fn test_guard_rules_login_required() {
    let rules = GuardRules::login_required();
    assert!(rules.login_required);
    assert!(rules.roles.is_empty());
}

#[test]
fn test_guard_rules_role_simple() {
    let rules = GuardRules::role("admin");
    assert!(!rules.login_required);
    assert_eq!(rules.roles, vec!["admin".to_string()]);
}

#[test]
fn test_guard_rules_plusieurs_roles() {
    let rules = GuardRules::roles(["admin", "moderateur"]);
    assert!(!rules.login_required);
    assert_eq!(rules.roles.len(), 2);
    assert!(rules.roles.contains(&"admin".to_string()));
    assert!(rules.roles.contains(&"moderateur".to_string()));
}

#[test]
fn test_guard_rules_login_et_role() {
    let rules = GuardRules::login_and_role("admin");
    assert!(rules.login_required);
    assert_eq!(rules.roles, vec!["admin".to_string()]);
}

#[test]
fn test_guard_rules_login_et_plusieurs_roles() {
    let rules = GuardRules::login_and_roles(["admin", "superuser"]);
    assert!(rules.login_required);
    assert_eq!(rules.roles.len(), 2);
}

#[test]
fn test_guard_rules_with_role_ajoute() {
    let rules = GuardRules::login_required().with_role("admin");
    assert!(rules.login_required);
    assert!(rules.roles.contains(&"admin".to_string()));
}

#[test]
fn test_guard_rules_with_role_chaine() {
    let rules = GuardRules::default()
        .with_role("editeur")
        .with_role("moderateur");
    assert_eq!(rules.roles.len(), 2);
}

// ═══════════════════════════════════════════════════════════════
// evaluate_rules
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_evaluate_pas_de_regles_passe_sans_ctx() {
    let rules = GuardRules::default();
    assert!(evaluate_rules(&rules, None).is_ok());
}

#[test]
fn test_evaluate_pas_de_regles_passe_avec_ctx() {
    let rules = GuardRules::default();
    let ctx = GuardContext::default();
    assert!(evaluate_rules(&rules, Some(&ctx)).is_ok());
}

#[test]
fn test_evaluate_login_requis_sans_ctx_refuse() {
    let rules = GuardRules::login_required();
    let result = evaluate_rules(&rules, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_evaluate_login_requis_non_authentifie_refuse() {
    let rules = GuardRules::login_required();
    let ctx = GuardContext::default(); // user_id = None
    let result = evaluate_rules(&rules, Some(&ctx));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_evaluate_login_requis_authentifie_passe() {
    let rules = GuardRules::login_required();
    let ctx = GuardContext {
        user_id: Some("user-1".to_string()),
        roles: vec![],
    };
    assert!(evaluate_rules(&rules, Some(&ctx)).is_ok());
}

#[test]
fn test_evaluate_role_requis_a_le_role_passe() {
    let rules = GuardRules::role("admin");
    let ctx = GuardContext {
        user_id: None,
        roles: vec!["admin".to_string()],
    };
    assert!(evaluate_rules(&rules, Some(&ctx)).is_ok());
}

#[test]
fn test_evaluate_role_requis_manque_le_role_refuse() {
    let rules = GuardRules::role("admin");
    let ctx = GuardContext {
        user_id: Some("user".to_string()),
        roles: vec!["editeur".to_string()],
    };
    let result = evaluate_rules(&rules, Some(&ctx));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().status(), StatusCode::FORBIDDEN);
}

#[test]
fn test_evaluate_role_requis_sans_ctx_refuse() {
    let rules = GuardRules::role("admin");
    let result = evaluate_rules(&rules, None);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().status(), StatusCode::FORBIDDEN);
}

#[test]
fn test_evaluate_login_et_role_authentifie_avec_role_passe() {
    let rules = GuardRules::login_and_role("admin");
    let ctx = GuardContext {
        user_id: Some("user".to_string()),
        roles: vec!["admin".to_string()],
    };
    assert!(evaluate_rules(&rules, Some(&ctx)).is_ok());
}

#[test]
fn test_evaluate_login_et_role_authentifie_sans_role_refuse() {
    let rules = GuardRules::login_and_role("admin");
    let ctx = GuardContext {
        user_id: Some("user".to_string()),
        roles: vec!["editeur".to_string()],
    };
    let result = evaluate_rules(&rules, Some(&ctx));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().status(), StatusCode::FORBIDDEN);
}

#[test]
fn test_evaluate_login_et_role_non_authentifie_refuse() {
    let rules = GuardRules::login_and_role("admin");
    let ctx = GuardContext {
        user_id: None,
        roles: vec!["admin".to_string()],
    };
    let result = evaluate_rules(&rules, Some(&ctx));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_evaluate_multi_roles_un_suffit() {
    let rules = GuardRules::roles(["admin", "moderateur"]);
    let ctx = GuardContext {
        user_id: Some("user".to_string()),
        roles: vec!["moderateur".to_string()],
    };
    assert!(evaluate_rules(&rules, Some(&ctx)).is_ok());
}
