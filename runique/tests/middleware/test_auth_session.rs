// Tests pour CurrentUser (auth_session)
use runique::admin::{Droit, Groupe};
use runique::middleware::auth::auth_session::CurrentUser;

fn make_user_with_droits(droits: Vec<Droit>) -> CurrentUser {
    CurrentUser {
        id: 1,
        username: "alice".to_string(),
        is_staff: true,
        is_superuser: false,
        droits,
        groupes: vec![],
    }
}

#[test]
fn test_current_user_has_droit() {
    let user = make_user_with_droits(vec![
        Droit {
            id: 1,
            nom: "editor".to_string(),
            resource_key: None,
            access_type: None,
        },
        Droit {
            id: 2,
            nom: "moderator".to_string(),
            resource_key: None,
            access_type: None,
        },
    ]);
    assert!(user.has_droit("editor"));
    assert!(user.has_any_droit(&["moderator", "admin"]));
    assert!(user.can_access_admin());
}

#[test]
fn test_current_user_no_droits() {
    let user = CurrentUser {
        id: 2,
        username: "bob".to_string(),
        is_staff: false,
        is_superuser: false,
        droits: vec![],
        groupes: vec![],
    };
    assert!(!user.has_droit("admin"));
    assert!(!user.has_any_droit(&["admin", "editor"]));
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
        droits: vec![],
        groupes: vec![],
    };
    assert!(user.can_access_admin());
    assert!(user.can_admin(&["anyrole"])); // superuser bypass
}

#[test]
fn test_current_user_can_admin_droits() {
    let user = CurrentUser {
        id: 4,
        username: "mod".to_string(),
        is_staff: false,
        is_superuser: false,
        droits: vec![Droit {
            id: 1,
            nom: "moderator".to_string(),
            resource_key: None,
            access_type: None,
        }],
        groupes: vec![],
    };
    assert!(user.can_admin(&["moderator"]));
    assert!(!user.can_admin(&["admin"]));
}

#[test]
fn test_current_user_droits_effectifs_merge_groupes() {
    let user = CurrentUser {
        id: 5,
        username: "dev".to_string(),
        is_staff: false,
        is_superuser: false,
        droits: vec![Droit {
            id: 1,
            nom: "read".to_string(),
            resource_key: None,
            access_type: None,
        }],
        groupes: vec![Groupe {
            id: 1,
            nom: "writers".to_string(),
            droits: vec![Droit {
                id: 2,
                nom: "write".to_string(),
                resource_key: None,
                access_type: None,
            }],
        }],
    };
    let effectifs = user.droits_effectifs();
    assert!(effectifs.iter().any(|d| d.nom == "read"));
    assert!(effectifs.iter().any(|d| d.nom == "write"));
}
