// Tests pour CurrentUser (auth_session)
use crate::helpers::pk::pk;
use runique::auth::current_user::CurrentUser;

#[test]
fn test_current_user_superuser() {
    let user = CurrentUser {
        id: pk(3),
        username: "root".to_string(),
        rang: runique::auth::permissions::rang::Rang::Superuser,
        groupes: vec![],
        droits: Vec::new(),
    };
    assert!(user.can_access_admin());
    assert!(user.can_access_resource("any")); // superuser bypass
}
