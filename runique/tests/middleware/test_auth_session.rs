// Tests pour CurrentUser (auth_session)
#[test]

use runique::middleware::auth::auth_session::CurrentUser;

#[test]
fn test_current_user_roles() {
    let user = CurrentUser {
        id: 1,
        username: "alice".to_string(),
        is_staff: true,
        is_superuser: false,
        roles: vec!["editor".to_string(), "moderator".to_string()],
    };
    assert!(user.has_role("editor"));
    assert!(user.has_any_role(&["moderator", "admin"]));
    assert!(user.can_access_admin());
}

#[test]
fn test_current_user_no_roles() {
    let user = CurrentUser {
        id: 2,
        username: "bob".to_string(),
        is_staff: false,
        is_superuser: false,
        roles: vec![],
    };
    assert!(!user.has_role("admin"));
    assert!(!user.has_any_role(&["admin", "editor"]));
    assert!(!user.can_access_admin());
    assert!(!user.can_admin(&["admin"]));
}

#[test]
fn test_current_user_superuser() {
    let user = CurrentUser {
        id: 3,
        username: "root".to_string(),
        is_staff: false,
        is_superuser: true,
        roles: vec![],
    };
    assert!(user.can_access_admin());
    assert!(user.can_admin(&["anyrole"])); // superuser bypass
}

#[test]
fn test_current_user_can_admin_roles() {
    let user = CurrentUser {
        id: 4,
        username: "mod".to_string(),
        is_staff: false,
        is_superuser: false,
        roles: vec!["moderator".to_string()],
    };
    assert!(user.can_admin(&["moderator"]));
    assert!(!user.can_admin(&["admin"]));
}
