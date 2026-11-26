use crate::migration::tests::*;

// ── Changes::is_empty ──

#[test]
fn empty_changes_is_empty() {
    assert!(empty_changes().is_empty());
}

#[test]
fn new_table_is_not_empty() {
    let c = Changes {
        table: crate::migration::TableChange::Created,
        ..empty_changes()
    };
    assert!(!c.is_empty(), "a new table is a change");
}

#[test]
fn a_lone_rename_is_not_empty() {
    let c = Changes {
        renames: Default::default(),
        renamed_columns: vec![("a".into(), "b".into())],
        ..empty_changes()
    };
    assert!(!c.is_empty(), "a rename is a change");
}
