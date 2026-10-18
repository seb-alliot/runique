//! Tests — CurrentUser
//! Couvre : permissions_effectives, can_access_resource, can_access_admin

use crate::helpers::pk::pk;
use runique::auth::permissions::{Groupe, Permission};
use runique::auth::rang::Rang;
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

/// `is_staff`: whether the account's only group opens the admin.
fn user(is_staff: bool, rang: Rang, permissions_v: Vec<Permission>) -> CurrentUser {
    CurrentUser {
        id: pk(1),
        username: "alice".to_string(),
        rang,
        groupes: vec![Groupe {
            id: 1,
            nom: "test_group".into(),
            is_staff,
            permissions: permissions_v,
        }],
    }
}

#[test]
fn test_can_access_resource_matching() {
    let u = user(
        false,
        Rang::Lambda,
        vec![perm("articles", false, true, false, false)],
    );
    assert!(u.can_access_resource("articles"));
}

#[test]
fn test_can_access_resource_no_read() {
    let u = user(
        false,
        Rang::Lambda,
        vec![perm("articles", true, false, false, false)],
    );
    assert!(!u.can_access_resource("articles")); // need can_read
}

#[test]
fn test_can_access_resource_missing() {
    let u = user(
        false,
        Rang::Lambda,
        vec![perm("users", false, true, false, false)],
    );
    assert!(!u.can_access_resource("articles"));
}

#[test]
fn test_superuser_can_access_anything() {
    let u = user(false, Rang::Superuser, vec![]);
    assert!(u.can_access_resource("articles"));
    assert!(u.can_access_resource("anything"));
}

#[test]
fn test_can_access_admin() {
    assert!(user(true, Rang::Lambda, vec![]).can_access_admin());
    assert!(!user(false, Rang::Lambda, vec![]).can_access_admin());
    for rang in [Rang::Admin, Rang::Dirigeant, Rang::Superuser] {
        assert!(user(false, rang, vec![]).can_access_admin(), "{rang:?}");
    }
}

#[test]
fn test_is_staff_needs_one_staff_group() {
    let mut u = user(false, Rang::Lambda, vec![]);
    assert!(!u.is_staff());
    u.groupes.push(Groupe {
        id: 2,
        nom: "equipe".into(),
        is_staff: true,
        permissions: vec![],
    });
    assert!(u.is_staff());
    u.groupes.clear();
    assert!(!u.is_staff(), "no group, no staff");
}

#[test]
fn test_only_the_superuser_reads_everything() {
    for rang in [Rang::Lambda, Rang::Admin, Rang::Dirigeant] {
        assert!(
            !user(true, rang, vec![]).can_access_resource("articles"),
            "{rang:?}"
        );
    }
}

#[test]
fn test_permissions_effectives_merge() {
    let u = CurrentUser {
        id: pk(1),
        username: "".into(),
        rang: runique::auth::rang::Rang::Lambda,
        groupes: vec![
            Groupe {
                id: 1,
                nom: "g1".into(),
                is_staff: true,
                permissions: vec![perm("articles", true, false, false, false)],
            },
            Groupe {
                id: 2,
                nom: "g2".into(),
                is_staff: true,
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
