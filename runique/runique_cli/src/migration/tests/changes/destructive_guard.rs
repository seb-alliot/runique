use crate::migration::tests::*;

// ── Destructive guard ──

#[test]
fn cascade_fk_on_new_table_is_not_destructive() {
    let changes = vec![change(true, vec![cascade_fk()], vec![])];
    assert!(
        collect_destructive_messages(&changes).is_empty(),
        "CASCADE FK on a brand-new table must not be flagged destructive"
    );
}

#[test]
fn cascade_fk_on_existing_table_is_destructive() {
    let changes = vec![change(false, vec![cascade_fk()], vec![])];
    let msgs = collect_destructive_messages(&changes);
    assert!(
        msgs.iter().any(|m| m.contains("CASCADE")),
        "CASCADE FK on an existing table must be flagged: {msgs:?}"
    );
}

#[test]
fn drop_column_is_destructive() {
    let dropped = vec![ParsedColumn {
        name: "removed".into(),
        ..Default::default()
    }];
    let changes = vec![change(false, vec![], dropped)];
    let msgs = collect_destructive_messages(&changes);
    assert!(
        msgs.iter().any(|m| m.contains("DROP COLUMN")),
        "dropping a column must be flagged: {msgs:?}"
    );
}
