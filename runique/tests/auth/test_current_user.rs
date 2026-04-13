//! Tests — CurrentUser
//! Couvre : permissions_effectives, can_access_resource, can_access_admin

use runique::admin::{Groupe, permissions::Permission};
use runique::auth::session::CurrentUser;

fn perm(resource: &str, create: bool, read: bool, update: bool, delete: bool) -> Permission {
    Permission {
        resource_key: resource.to_string(),
        can_create: create,
        can_read: read,
        can_update: update,
        can_delete: delete,
        can_update_own: false,
        can_delete_own: false,
    }
}

fn user(is_staff: bool, is_superuser: bool, permissions_v: Vec<Permission>) -> CurrentUser {
    CurrentUser {
        id: 1,
        username: "alice".to_string(),
        is_staff,
        is_superuser,
        groupes: vec![Groupe {
            id: 1,
            nom: "test_group".into(),
            permissions: permissions_v,
        }],
    }
}

#[test]
fn test_can_access_resource_matching() {
    let u = user(
        false,
        false,
        vec![perm("articles", false, true, false, false)],
    );
    assert!(u.can_access_resource("articles"));
}

#[test]
fn test_can_access_resource_no_read() {
    let u = user(
        false,
        false,
        vec![perm("articles", true, false, false, false)],
    );
    assert!(!u.can_access_resource("articles")); // need can_read
}

#[test]
fn test_can_access_resource_missing() {
    let u = user(false, false, vec![perm("users", false, true, false, false)]);
    assert!(!u.can_access_resource("articles"));
}

#[test]
fn test_superuser_can_access_anything() {
    let u = user(false, true, vec![]);
    assert!(u.can_access_resource("articles"));
    assert!(u.can_access_resource("anything"));
}

#[test]
fn test_can_access_admin() {
    assert!(user(true, false, vec![]).can_access_admin());
    assert!(user(false, true, vec![]).can_access_admin());
    assert!(!user(false, false, vec![]).can_access_admin());
}

#[test]
fn test_permissions_effectives_merge() {
    let u = CurrentUser {
        id: 1,
        username: "".into(),
        is_staff: false,
        is_superuser: false,
        groupes: vec![
            Groupe {
                id: 1,
                nom: "g1".into(),
                permissions: vec![perm("articles", true, false, false, false)],
            },
            Groupe {
                id: 2,
                nom: "g2".into(),
                permissions: vec![perm("articles", false, true, false, false)],
            },
        ],
    };

    let effectifs = u.permissions_effectives();
    assert_eq!(effectifs.len(), 1);
    let p = &effectifs[0];
    assert!(p.can_create);
    assert!(p.can_read);
    assert!(!p.can_update);
}
