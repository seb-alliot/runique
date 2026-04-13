//! Tests unitaires — cache des permissions (LazyLock + RwLock).
//!
//! Ces tests vérifient le comportement du cache mémoire sans DB.
//! Attention : le cache est global (LazyLock statique) — chaque test
//! doit utiliser des `user_id` distincts pour éviter les interférences.

use runique::admin::permissions::{Groupe, Permission};
use runique::auth::guard::{cache_permissions, clear_cache, evict_permissions, get_permissions};

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
// cache_permissions / get_permissions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_cache_puis_get_retourne_valeur() {
    let user_id = 10_001;
    cache_permissions(user_id, vec![make_groupe("articles", true)]);
    let cached = get_permissions(user_id);
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().groupes.len(), 1);
}

#[test]
fn test_get_user_inconnu_retourne_none() {
    let user_id = 10_002;
    // Aucune entrée pour cet user
    assert!(get_permissions(user_id).is_none());
}

#[test]
fn test_cache_ecrase_entree_existante() {
    let user_id = 10_003;
    cache_permissions(user_id, vec![make_groupe("articles", true)]);
    // Écrase avec des droits différents
    cache_permissions(user_id, vec![make_groupe("articles", false)]);
    let cached = get_permissions(user_id).unwrap();
    assert!(!cached.groupes[0].permissions[0].can_read);
}

// ═══════════════════════════════════════════════════════════════
// evict_permissions
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_evict_retire_user_du_cache() {
    let user_id = 10_004;
    cache_permissions(user_id, vec![make_groupe("articles", true)]);
    assert!(get_permissions(user_id).is_some());
    evict_permissions(user_id);
    assert!(get_permissions(user_id).is_none());
}

#[test]
fn test_evict_user_inconnu_ne_panique_pas() {
    evict_permissions(99_999); // ne doit pas paniquer
}

// ═══════════════════════════════════════════════════════════════
// clear_cache
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_clear_cache_vide_tous_les_users() {
    let u1 = 10_005;
    let u2 = 10_006;
    cache_permissions(u1, vec![make_groupe("articles", true)]);
    cache_permissions(u2, vec![make_groupe("users", true)]);
    clear_cache();
    assert!(get_permissions(u1).is_none());
    assert!(get_permissions(u2).is_none());
}

#[test]
fn test_clear_cache_puis_reinsert_fonctionne() {
    let user_id = 10_007;
    cache_permissions(user_id, vec![make_groupe("articles", true)]);
    clear_cache();
    assert!(get_permissions(user_id).is_none());
    // On peut réinsérer après clear
    cache_permissions(user_id, vec![make_groupe("articles", false)]);
    assert!(get_permissions(user_id).is_some());
}

// ═══════════════════════════════════════════════════════════════
// Agrégation multi-groupes (via CurrentUser)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_multi_groupes_or_permissions() {
    use runique::admin::permissions::Groupe;
    use runique::auth::session::CurrentUser;

    let user = CurrentUser {
        id: 10_008,
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
        id: 10_009,
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
        id: 10_010,
        username: "carol".into(),
        is_staff: true,
        is_superuser: false,
        groupes: vec![make_groupe("articles", true)], // can_read = true
    };
    assert!(user.can_access_resource("articles"));
}
