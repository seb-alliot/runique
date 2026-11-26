use crate::migration::tests::*;

// ── Foreign key actions (inline in the CREATE file) ──

fn fk_schema(table: &str, fk: ParsedFk) -> ParsedSchema {
    ParsedSchema {
        table_name: table.into(),
        primary_key: None,
        columns: vec![],
        foreign_keys: vec![fk],
        indexes: vec![],
    }
}

#[test]
fn fk_cascade_action_is_rendered() {
    let sql = generate_create_file(&fk_schema(
        "comment",
        ParsedFk {
            from_column: "post_id".into(),
            to_table: "post".into(),
            to_column: "id".into(),
            on_delete: crate::migration::FkAction::Cascade,
            on_update: crate::migration::FkAction::NoAction,
        },
    ));
    assert!(
        sql.contains(".on_delete(ForeignKeyAction::Cascade)"),
        "{sql}"
    );
    assert!(
        sql.contains(r#".from(Alias::new("comment"), Alias::new("post_id"))"#),
        "{sql}"
    );
    assert!(
        sql.contains(r#".to(Alias::new("post"), Alias::new("id"))"#),
        "{sql}"
    );
}

#[test]
fn fk_set_null_and_restrict_actions_render() {
    let sql = generate_create_file(&fk_schema(
        "child",
        ParsedFk {
            from_column: "a_id".into(),
            to_table: "a".into(),
            to_column: "id".into(),
            on_delete: crate::migration::FkAction::SetNull,
            on_update: crate::migration::FkAction::Restrict,
        },
    ));
    assert!(
        sql.contains(".on_delete(ForeignKeyAction::SetNull)"),
        "{sql}"
    );
    assert!(
        sql.contains(".on_update(ForeignKeyAction::Restrict)"),
        "{sql}"
    );
}
