//! Tests — rights of a `CurrentUser` aggregated across groups.

use crate::helpers::pk::pk;
use runique::auth::permissions::{Groupe, Permission};

fn make_groupe(resource: &str, can_read: bool) -> Groupe {
    Groupe {
        id: 1,
        nom: "test".to_string(),
        permissions: vec![Permission {
            resource_key: resource.to_string(),
            can_create: false,
            can_read,
            can_update: false,
            can_delete: false,
            can_update_own: false,
            can_delete_own: false,
        }],
    }
}

// ═══════════════════════════════════════════════════════════════
// Agrégation multi-groupes (via CurrentUser)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_multi_groupes_or_permissions() {
    use runique::auth::permissions::Groupe;
    use runique::auth::session::CurrentUser;

    let user = CurrentUser {
        id: pk(10_008),
        username: "alice".into(),
        is_staff: true,
        is_superuser: false,
        groupes: vec![
            Groupe {
                id: 1,
                nom: "moderateur".into(),
                permissions: vec![Permission {
                    resource_key: "articles".to_string(),
                    can_create: false,
                    can_read: true,
                    can_update: false,
                    can_delete: false,
                    can_update_own: false,
                    can_delete_own: false,
                }],
            },
            Groupe {
                id: 2,
                nom: "editeur".into(),
                permissions: vec![Permission {
                    resource_key: "articles".to_string(),
                    can_create: true,
                    can_read: false,
                    can_update: true,
                    can_delete: false,
                    can_update_own: false,
                    can_delete_own: false,
                }],
            },
        ],
    };

    let effectifs = user.permissions_effectives();
    assert_eq!(effectifs.len(), 1);
    let p = &effectifs[0];
    // OR des deux groupes
    assert!(p.can_create);
    assert!(p.can_read);
    assert!(p.can_update);
    assert!(!p.can_delete);
}

#[test]
fn test_can_access_resource_necessite_can_read() {
    use runique::auth::session::CurrentUser;

    let user = CurrentUser {
        id: pk(10_009),
        username: "bob".into(),
        is_staff: true,
        is_superuser: false,
        groupes: vec![make_groupe("articles", false)], // can_read = false
    };
    assert!(!user.can_access_resource("articles"));
}

#[test]
fn test_can_access_resource_avec_can_read() {
    use runique::auth::session::CurrentUser;

    let user = CurrentUser {
        id: pk(10_010),
        username: "carol".into(),
        is_staff: true,
        is_superuser: false,
        groupes: vec![make_groupe("articles", true)], // can_read = true
    };
    assert!(user.can_access_resource("articles"));
}
