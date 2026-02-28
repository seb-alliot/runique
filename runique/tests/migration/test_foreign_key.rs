// Tests pour ForeignKeyDef

use runique::migration::foreign_key::ForeignKeyDef;
use sea_query::ForeignKeyAction;

// ═══════════════════════════════════════════════════════════════
// Valeurs par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_foreign_key_new_defauts() {
    let fk = ForeignKeyDef::new("user_id");
    assert_eq!(fk.from_column, "user_id");
    assert_eq!(fk.to_column, "id", "to_column doit être 'id' par défaut");
    assert!(fk.to_table.is_empty());
    assert!(matches!(fk.on_delete, ForeignKeyAction::NoAction));
    assert!(matches!(fk.on_update, ForeignKeyAction::NoAction));
}

// ═══════════════════════════════════════════════════════════════
// Builder — references, to_column
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_foreign_key_references() {
    let fk = ForeignKeyDef::new("user_id").references("users");
    assert_eq!(fk.to_table, "users");
}

#[test]
fn test_foreign_key_to_column_personnalise() {
    let fk = ForeignKeyDef::new("author_id")
        .references("authors")
        .to_column("uid");
    assert_eq!(fk.to_column, "uid");
}

// ═══════════════════════════════════════════════════════════════
// Builder — on_delete, on_update
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_foreign_key_on_delete_cascade() {
    let fk = ForeignKeyDef::new("post_id").on_delete(ForeignKeyAction::Cascade);
    assert!(matches!(fk.on_delete, ForeignKeyAction::Cascade));
}

#[test]
fn test_foreign_key_on_update_set_null() {
    let fk = ForeignKeyDef::new("category_id").on_update(ForeignKeyAction::SetNull);
    assert!(matches!(fk.on_update, ForeignKeyAction::SetNull));
}

#[test]
fn test_foreign_key_on_delete_restrict() {
    let fk = ForeignKeyDef::new("tag_id").on_delete(ForeignKeyAction::Restrict);
    assert!(matches!(fk.on_delete, ForeignKeyAction::Restrict));
}

// ═══════════════════════════════════════════════════════════════
// Builder complet (chaîné)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_foreign_key_builder_complet() {
    let fk = ForeignKeyDef::new("owner_id")
        .references("users")
        .to_column("id")
        .on_delete(ForeignKeyAction::Cascade)
        .on_update(ForeignKeyAction::NoAction);
    assert_eq!(fk.from_column, "owner_id");
    assert_eq!(fk.to_table, "users");
    assert_eq!(fk.to_column, "id");
    assert!(matches!(fk.on_delete, ForeignKeyAction::Cascade));
    assert!(matches!(fk.on_update, ForeignKeyAction::NoAction));
}

// ═══════════════════════════════════════════════════════════════
// Clone + to_sea_foreign_key (ne panique pas)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_foreign_key_clone() {
    let fk = ForeignKeyDef::new("user_id")
        .references("users")
        .on_delete(ForeignKeyAction::Cascade);
    let cloned = fk.clone();
    assert_eq!(cloned.from_column, "user_id");
    assert_eq!(cloned.to_table, "users");
}

#[test]
fn test_foreign_key_to_sea_foreign_key_compile() {
    let fk = ForeignKeyDef::new("user_id")
        .references("users")
        .on_delete(ForeignKeyAction::Cascade);
    let _ = fk.to_sea_foreign_key("posts");
}
