use crate::migration::tests::*;

// ── ALTER: add / drop / modify column (up + down) ──

#[test]
fn alter_add_column_up_adds_down_drops() {
    let changes = Changes {
        added_columns: vec![ParsedColumn {
            name: "nickname".into(),
            col_type: crate::migration::ColumnType::String,
            nullable: true,
            ..Default::default()
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#".add_column(ColumnDef::new(Alias::new("nickname")).string().null())"#),
        "up:\n{up}"
    );
    assert!(
        down.contains(r#".drop_column(Alias::new("nickname"))"#),
        "down:\n{down}"
    );
}

#[test]
fn alter_drop_column_up_drops_down_recreates() {
    let changes = Changes {
        dropped_columns: vec![ParsedColumn {
            name: "legacy".into(),
            col_type: crate::migration::ColumnType::Integer,
            ..Default::default()
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#".drop_column(Alias::new("legacy"))"#),
        "up:\n{up}"
    );
    assert!(
        down.contains(r#".add_column(ColumnDef::new(Alias::new("legacy")).integer()"#),
        "down must recreate the dropped column:\n{down}"
    );
}

#[test]
fn alter_type_change_emits_warning_not_modify() {
    // A type change is never auto-applied (risk of data loss) — it's a manual WARNING.
    let old = ParsedColumn {
        name: "age".into(),
        col_type: crate::migration::ColumnType::Integer,
        ..Default::default()
    };
    let new = ParsedColumn {
        name: "age".into(),
        col_type: crate::migration::ColumnType::BigInteger,
        ..Default::default()
    };
    let changes = Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    assert!(
        sql.contains("WARNING: type change on column 'age': Integer -> BigInteger"),
        "type change must emit a manual WARNING:\n{sql}"
    );
    assert!(
        !sql.contains(".modify_column("),
        "type change must NOT auto-modify:\n{sql}"
    );
}

#[test]
fn alter_type_change_only_body_uses_underscored_manager() {
    // A type change emits only a `// WARNING` comment — no real statement touches `manager`.
    // The generated fn must take `_manager`, otherwise the file fails to compile clean
    // (`unused_variable: manager`).
    let old = ParsedColumn {
        name: "age".into(),
        col_type: crate::migration::ColumnType::Integer,
        ..Default::default()
    };
    let new = ParsedColumn {
        name: "age".into(),
        col_type: crate::migration::ColumnType::BigInteger,
        ..Default::default()
    };
    let changes = Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    assert!(
        sql.contains("async fn up(&self, _manager: &SchemaManager)"),
        "comment-only up body must use _manager:\n{sql}"
    );
    assert!(
        sql.contains("async fn down(&self, _manager: &SchemaManager)"),
        "empty down body must use _manager:\n{sql}"
    );
}

#[test]
fn alter_nullable_to_not_null_modifies_and_reverses() {
    // Same type, nullable → not_null is applied via modify_column (down restores nullable).
    let old = ParsedColumn {
        name: "bio".into(),
        col_type: crate::migration::ColumnType::String,
        nullable: true,
        ..Default::default()
    };
    let new = ParsedColumn {
        name: "bio".into(),
        col_type: crate::migration::ColumnType::String,
        nullable: false,
        ..Default::default()
    };
    let changes = Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#".modify_column(ColumnDef::new(Alias::new("bio")).string().not_null())"#),
        "up:\n{up}"
    );
    assert!(
        down.contains(r#".modify_column(ColumnDef::new(Alias::new("bio")).string().null())"#),
        "down must restore nullable:\n{down}"
    );
}
