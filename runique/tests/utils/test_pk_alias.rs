// Tests pour l'alias global `Pk` (runique::utils::config::Pk) — doit suivre
// big-pk/pk-uuid, source de vérité unique pour le type de PK de l'application.

use runique::utils::config::Pk;

#[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
#[test]
fn test_pk_alias_is_i32_by_default() {
    let value: Pk = 1i32;
    assert_eq!(value, 1i32);
}

#[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
#[test]
fn test_pk_alias_is_i64_under_big_pk() {
    let value: Pk = 1i64;
    assert_eq!(value, 1i64);
}

#[cfg(feature = "pk-uuid")]
#[test]
fn test_pk_alias_is_uuid_under_pk_uuid() {
    let value: Pk = uuid::Uuid::now_v7();
    assert_eq!(value.get_version_num(), 7);
}
