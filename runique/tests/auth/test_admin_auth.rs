// Tests pour AdminLoginResult

use runique::middleware::auth::AdminLoginResult;

// ═══════════════════════════════════════════════════════════════
// AdminLoginResult — accès aux champs
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_admin_login_result_champs_basiques() {
    let result = AdminLoginResult {
        user_id: 42,
        username: "admin".to_string(),
        is_staff: true,
        is_superuser: false,
        roles: vec!["editor".to_string()],
    };
    assert_eq!(result.user_id, 42);
    assert_eq!(result.username, "admin");
    assert!(result.is_staff);
    assert!(!result.is_superuser);
    assert_eq!(result.roles, vec!["editor"]);
}

#[test]
fn test_admin_login_result_superuser() {
    let result = AdminLoginResult {
        user_id: 1,
        username: "root".to_string(),
        is_staff: false,
        is_superuser: true,
        roles: vec![],
    };
    assert!(!result.is_staff);
    assert!(result.is_superuser);
    assert!(result.roles.is_empty());
}

#[test]
fn test_admin_login_result_roles_multiples() {
    let result = AdminLoginResult {
        user_id: 3,
        username: "mod".to_string(),
        is_staff: true,
        is_superuser: false,
        roles: vec![
            "editor".to_string(),
            "moderator".to_string(),
            "writer".to_string(),
        ],
    };
    assert_eq!(result.roles.len(), 3);
    assert!(result.roles.contains(&"editor".to_string()));
    assert!(result.roles.contains(&"moderator".to_string()));
}

// ═══════════════════════════════════════════════════════════════
// AdminLoginResult — Clone
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_admin_login_result_clone() {
    let result = AdminLoginResult {
        user_id: 7,
        username: "alice".to_string(),
        is_staff: true,
        is_superuser: true,
        roles: vec!["admin".to_string()],
    };
    let cloned = result.clone();
    assert_eq!(cloned.user_id, result.user_id);
    assert_eq!(cloned.username, result.username);
    assert_eq!(cloned.is_staff, result.is_staff);
    assert_eq!(cloned.is_superuser, result.is_superuser);
    assert_eq!(cloned.roles, result.roles);
}

// ═══════════════════════════════════════════════════════════════
// AdminLoginResult — Debug
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_admin_login_result_debug_contient_username() {
    let result = AdminLoginResult {
        user_id: 1,
        username: "debug_user".to_string(),
        is_staff: false,
        is_superuser: false,
        roles: vec![],
    };
    let debug = format!("{:?}", result);
    assert!(debug.contains("debug_user"));
    assert!(debug.contains("user_id"));
}
