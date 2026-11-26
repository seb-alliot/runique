use crate::migration::tests::*;

// ── ALTER: multiple added columns ──

#[test]
fn alter_multiple_added_columns_all_present() {
    let changes = Changes {
        added_columns: vec![
            ParsedColumn {
                name: "a".into(),
                col_type: crate::migration::ColumnType::Integer,
                nullable: true,
                ..Default::default()
            },
            ParsedColumn {
                name: "b".into(),
                col_type: crate::migration::ColumnType::String,
                nullable: true,
                ..Default::default()
            },
        ],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    assert!(
        sql.contains(r#".add_column(ColumnDef::new(Alias::new("a")).integer().null())"#),
        "{sql}"
    );
    assert!(
        sql.contains(r#".add_column(ColumnDef::new(Alias::new("b")).string().null())"#),
        "{sql}"
    );
}
