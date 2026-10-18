// Tests pour CurrentUser (auth_session)
use crate::helpers::pk::pk;
use runique::auth::session::CurrentUser;

#[test]
fn test_current_user_superuser() {
    let user = CurrentUser {
        id: pk(3),
        username: "root".to_string(),
        rang: runique::auth::rang::Rang::Superuser,
        groupes: vec![],
    };
    assert!(user.can_access_admin());
    assert!(user.can_access_resource("any")); // superuser bypass
}
