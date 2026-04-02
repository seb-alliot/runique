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
    };
    assert_eq!(result.user_id, 42);
    assert_eq!(result.username, "admin");
    assert!(result.is_staff);
    assert!(!result.is_superuser);
}

#[test]
fn test_admin_login_result_superuser() {
    let result = AdminLoginResult {
        user_id: 1,
        username: "root".to_string(),
        is_staff: false,
        is_superuser: true,
    };
    assert!(!result.is_staff);
    assert!(result.is_superuser);
}

#[test]
fn test_admin_login_result_staff() {
    let result = AdminLoginResult {
        user_id: 3,
        username: "mod".to_string(),
        is_staff: true,
        is_superuser: false,
    };
    assert!(result.is_staff);
    assert!(!result.is_superuser);
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
    };
    let cloned = result.clone();
    assert_eq!(cloned.user_id, result.user_id);
    assert_eq!(cloned.username, result.username);
    assert_eq!(cloned.is_staff, result.is_staff);
    assert_eq!(cloned.is_superuser, result.is_superuser);
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
    };
    let debug = format!("{:?}", result);
    assert!(debug.contains("debug_user"));
    assert!(debug.contains("user_id"));
}
