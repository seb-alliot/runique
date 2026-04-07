// Tests pour CurrentUser (auth_session)
use runique::middleware::auth::auth_session::CurrentUser;

#[test]
fn test_current_user_superuser() {
    let user = CurrentUser {
        id: 3,
        username: "root".to_string(),
        is_staff: false,
        is_superuser: true,
        groupes: vec![],
    };
    assert!(user.can_access_admin());
    assert!(user.can_access_resource("any")); // superuser bypass
}
