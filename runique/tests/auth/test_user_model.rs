//! Tests — middleware/auth/user.rs
//! Couvre : RuniqueUser trait

use runique::middleware::auth::user::Model;
use runique::middleware::auth::user_trait::RuniqueUser;

// ─── Helper ──────────────────────────────────────────────────────────────────

fn make_model() -> Model {
    Model {
        id: 1,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        password: "hashed".to_string(),
        is_active: true,
        is_staff: false,
        is_superuser: false,
        created_at: None,
        updated_at: None,
    }
}

// ═══════════════════════════════════════════════════════════════
// RuniqueUser trait
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_user_id() {
    let model = make_model();
    assert_eq!(model.user_id(), 1);
}

#[test]
fn test_runique_user_username() {
    let model = make_model();
    assert_eq!(model.username(), "alice");
}

#[test]
fn test_runique_user_email() {
    let model = make_model();
    assert_eq!(model.email(), "alice@example.com");
}

#[test]
fn test_runique_user_password_hash() {
    let model = make_model();
    assert_eq!(model.password_hash(), "hashed");
}

#[test]
fn test_runique_user_is_active() {
    let model = make_model();
    assert!(model.is_active());
}

#[test]
fn test_runique_user_is_staff_false() {
    let model = make_model();
    assert!(!model.is_staff());
}

#[test]
fn test_runique_user_is_superuser_false() {
    let model = make_model();
    assert!(!model.is_superuser());
}

#[test]
fn test_runique_user_is_superuser_true() {
    let mut model = make_model();
    model.is_superuser = true;
    assert!(model.is_superuser());
}

#[test]
fn test_runique_user_roles_default_vide() {
    // roles() retourne toujours vec![] (implémentation par défaut du trait)
    let model = make_model();
    assert!(model.roles().is_empty());
}

#[test]
fn test_runique_user_can_access_admin_false() {
    let model = make_model();
    assert!(!model.can_access_admin());
}

#[test]
fn test_runique_user_can_access_admin_staff() {
    let mut model = make_model();
    model.is_staff = true;
    assert!(model.can_access_admin());
}
