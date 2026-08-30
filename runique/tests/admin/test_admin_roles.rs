//! Tests — admin/helper/roles.rs
//!
//! Registre global des rôles admin déclarés via `admin!{}`, peuplé au boot par
//! le code généré (`register_roles`) et lu en lecture seule ensuite
//! (`get_roles`). Stockage dans un `static RwLock` partagé par tout le
//! process : une seule fonction de test ici — une deuxième risquerait une
//! race avec `cargo test` (parallèle par défaut) sur le même registre.

use runique::admin::helper::roles::{get_roles, register_roles};

#[test]
fn test_register_and_get_roles_roundtrip() {
    assert!(
        get_roles().is_empty(),
        "précondition : aucun autre test du binaire ne doit toucher ce registre global"
    );

    register_roles(vec!["editor".to_string(), "viewer".to_string()]);
    assert_eq!(
        get_roles(),
        vec!["editor".to_string(), "viewer".to_string()]
    );

    // Un second appel remplace entièrement le registre (pas un append).
    register_roles(vec!["admin".to_string()]);
    assert_eq!(get_roles(), vec!["admin".to_string()]);
}
