use crate::migration::tests::*;

// ── Multiple renames in one ALTER ──

#[test]
fn multiple_column_renames_all_emitted() {
    let changes = Changes {
        renames: Default::default(),
        renamed_columns: vec![
            ("first".into(), "given_name".into()),
            ("last".into(), "family_name".into()),
        ],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    assert!(
        sql.contains(r#".rename_column(Alias::new("first"), Alias::new("given_name"))"#),
        "{sql}"
    );
    assert!(
        sql.contains(r#".rename_column(Alias::new("last"), Alias::new("family_name"))"#),
        "{sql}"
    );
}
