// Tests pour PrimaryKeyDef

use runique::migration::primary_key::PrimaryKeyDef;
use sea_query::ColumnType;

// ═══════════════════════════════════════════════════════════════
// Valeurs par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_primary_key_new_defauts() {
    let pk = PrimaryKeyDef::new("id");
    assert_eq!(pk.name, "id");
    assert!(
        pk.auto_increment,
        "auto_increment doit être true par défaut"
    );
    assert!(matches!(pk.col_type, ColumnType::Integer));
}

// ═══════════════════════════════════════════════════════════════
// Types de colonne
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_primary_key_i32() {
    let pk = PrimaryKeyDef::new("id").i32();
    assert!(matches!(pk.col_type, ColumnType::Integer));
}

#[test]
fn test_primary_key_i64() {
    let pk = PrimaryKeyDef::new("id").i64();
    assert!(matches!(pk.col_type, ColumnType::BigInteger));
}

#[test]
fn test_primary_key_uuid() {
    let pk = PrimaryKeyDef::new("id").uuid();
    assert!(matches!(pk.col_type, ColumnType::Uuid));
    assert!(!pk.auto_increment, "uuid désactive auto_increment");
}

// ═══════════════════════════════════════════════════════════════
// auto_increment / no_auto_increment
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_primary_key_auto_increment() {
    let pk = PrimaryKeyDef::new("id").i64().auto_increment();
    assert!(pk.auto_increment);
}

#[test]
fn test_primary_key_no_auto_increment() {
    let pk = PrimaryKeyDef::new("id").i32().no_auto_increment();
    assert!(!pk.auto_increment);
}

// ═══════════════════════════════════════════════════════════════
// Clone + Debug
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_primary_key_clone() {
    let pk = PrimaryKeyDef::new("pk").i64().no_auto_increment();
    let cloned = pk.clone();
    assert_eq!(cloned.name, "pk");
    assert!(!cloned.auto_increment);
}

#[test]
fn test_primary_key_to_sea_column_compile() {
    // Vérifie que la génération ne panique pas
    let pk = PrimaryKeyDef::new("id").i32();
    let _ = pk.to_sea_column();
}
