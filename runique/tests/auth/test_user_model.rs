//! Tests — middleware/auth/user.rs
//! Couvre : Model::get_roles, Model::set_roles, RuniqueUser trait

use runique::middleware::auth::user::Model;
use runique::middleware::auth::user_trait::RuniqueUser;

// ─── Helper ──────────────────────────────────────────────────────────────────

fn make_model(roles: Option<&str>) -> Model {
    Model {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        password: "hashed".to_string(),
        is_active: true,
        is_staff: false,
        is_superuser: false,
        roles: roles.map(|s| s.to_string()),
        created_at: None,
        updated_at: None,
    }
}

// ═══════════════════════════════════════════════════════════════
// get_roles
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_get_roles_none_retourne_vide() {
    let model = make_model(None);
    assert!(model.get_roles().is_empty());
}

#[test]
fn test_get_roles_json_valide() {
    let model = make_model(Some(r#"["editor","moderator"]"#));
    let roles = model.get_roles();
    assert_eq!(roles, vec!["editor", "moderator"]);
}

#[test]
fn test_get_roles_json_invalide_retourne_vide() {
    let model = make_model(Some("not-json"));
    assert!(model.get_roles().is_empty());
}

#[test]
fn test_get_roles_liste_vide_json() {
    let model = make_model(Some("[]"));
    assert!(model.get_roles().is_empty());
}

#[test]
fn test_get_roles_un_seul_role() {
    let model = make_model(Some(r#"["admin"]"#));
    assert_eq!(model.get_roles(), vec!["admin"]);
}

// ═══════════════════════════════════════════════════════════════
// set_roles
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_set_roles_liste_vide_retourne_none() {
    let result = Model::set_roles(vec![]);
    assert!(result.is_none());
}

#[test]
fn test_set_roles_un_role() {
    let result = Model::set_roles(vec!["admin".to_string()]);
    assert!(result.is_some());
    let json: Vec<String> = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(json, vec!["admin"]);
}

#[test]
fn test_set_roles_plusieurs_roles() {
    let result = Model::set_roles(vec!["editor".to_string(), "moderator".to_string()]);
    assert!(result.is_some());
    let json: Vec<String> = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(json, vec!["editor", "moderator"]);
}

#[test]
fn test_set_roles_roundtrip_avec_get_roles() {
    let roles_in = vec!["writer".to_string(), "reviewer".to_string()];
    let serialized = Model::set_roles(roles_in.clone()).unwrap();
    let model = make_model(Some(&serialized));
    assert_eq!(model.get_roles(), roles_in);
}

// ═══════════════════════════════════════════════════════════════
// RuniqueUser trait
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_user_id() {
    let model = make_model(None);
    assert_eq!(model.user_id(), 1);
}

#[test]
fn test_runique_user_username() {
    let model = make_model(None);
    assert_eq!(model.username(), "alice");
}

#[test]
fn test_runique_user_email() {
    let model = make_model(None);
    assert_eq!(model.email(), "alice@example.com");
}

#[test]
fn test_runique_user_password_hash() {
    let model = make_model(None);
    assert_eq!(model.password_hash(), "hashed");
}

#[test]
fn test_runique_user_is_active() {
    let model = make_model(None);
    assert!(model.is_active());
}

#[test]
fn test_runique_user_is_staff_false() {
    let model = make_model(None);
    assert!(!model.is_staff());
}

#[test]
fn test_runique_user_is_superuser_false() {
    let model = make_model(None);
    assert!(!model.is_superuser());
}

#[test]
fn test_runique_user_is_superuser_true() {
    let mut model = make_model(None);
    model.is_superuser = true;
    assert!(model.is_superuser());
}

#[test]
fn test_runique_user_roles_via_trait() {
    let model = make_model(Some(r#"["editor"]"#));
    assert_eq!(model.roles(), vec!["editor"]);
}

#[test]
fn test_runique_user_roles_vide_via_trait() {
    let model = make_model(None);
    assert!(model.roles().is_empty());
}
