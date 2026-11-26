use crate::migration::tests::*;

// ── ALTER: add UNIQUE via safe modify ──

#[test]
fn alter_add_unique_via_modify_column() {
    // Same type, same nullability, unique false → true → applied via modify_column.
    let old = ParsedColumn {
        name: "email".into(),
        col_type: crate::migration::ColumnType::String,
        nullable: false,
        unique: false,
        ..Default::default()
    };
    let new = ParsedColumn {
        name: "email".into(),
        col_type: crate::migration::ColumnType::String,
        nullable: false,
        unique: true,
        ..Default::default()
    };
    let changes = Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let (up, down) = up_down(&sql);
    assert!(
        up.contains(r#".modify_column(ColumnDef::new(Alias::new("email")).string().not_null().unique_key())"#),
        "up:\n{up}"
    );
    assert!(
        down.contains(r#".modify_column(ColumnDef::new(Alias::new("email")).string().not_null())"#),
        "down must drop the unique:\n{down}"
    );
}
