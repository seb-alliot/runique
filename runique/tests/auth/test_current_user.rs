//! Tests — CurrentUser
//! Couvre : rights_on (union des groupes), can_access_resource, can_access_admin

use crate::helpers::pk::pk;
use runique::auth::current_user::CurrentUser;
use runique::auth::permissions::rang::Rang;
use runique::auth::permissions::{Groupe, Permission};

fn perm(resource: &str, create: bool, read: bool, update: bool, delete: bool) -> Permission {
    Permission {
        resource_key: resource.to_string(),
        can_create: create,
        can_read: read,
        can_read_own: false,
        can_update: update,
        can_delete: delete,
        can_update_own: false,
        can_delete_own: false,
        colonnes: Vec::new(),
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
        droits: Vec::new(),
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

/// The rank decides: the dirigeant reads everything like the superuser; an
/// admin without his own grid and a lambda without a group read nothing.
#[test]
fn test_the_rank_decides_who_reads_without_rights() {
    for rang in [Rang::Superuser, Rang::Dirigeant] {
        assert!(
            user(false, rang, vec![]).can_access_resource("articles"),
            "{rang:?}"
        );
    }
    for rang in [Rang::Lambda, Rang::Admin] {
        assert!(
            !user(true, rang, vec![]).can_access_resource("articles"),
            "{rang:?}"
        );
    }
}

/// An admin reads by his own grid, never by his groups.
#[test]
fn test_an_admin_reads_by_his_grid_not_his_groups() {
    let mut admin = user(
        false,
        Rang::Admin,
        vec![perm("articles", false, true, false, false)],
    );
    assert!(!admin.can_access_resource("articles"), "group ignored");
    admin.droits = vec![perm("articles", false, true, false, false)];
    assert!(admin.can_access_resource("articles"), "his grid");
}

#[test]
fn test_rights_on_merges_the_groups() {
    let u = CurrentUser {
        id: pk(1),
        username: "".into(),
        rang: runique::auth::permissions::rang::Rang::Lambda,
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
        droits: Vec::new(),
    };

    let p = u.rights_on("articles").crud;
    assert!(p.can_create);
    assert!(p.can_read);
    assert!(!p.can_update);
}
