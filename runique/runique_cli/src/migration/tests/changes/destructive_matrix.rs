use crate::migration::tests::*;

// ── Destructive guard matrix ──

#[test]
fn destructive_type_change_is_flagged() {
    let old = ParsedColumn {
        name: "age".into(),
        col_type: crate::migration::ColumnType::Integer,
        ..Default::default()
    };
    let new = ParsedColumn {
        name: "age".into(),
        col_type: crate::migration::ColumnType::String,
        ..Default::default()
    };
    let changes = vec![Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    }];
    let msgs = collect_destructive_messages(&changes);
    assert!(
        msgs.iter().any(|m| m.contains("type Integer -> String")),
        "{msgs:?}"
    );
}

#[test]
fn destructive_nullable_to_not_null_is_flagged() {
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
    let changes = vec![Changes {
        modified_columns: vec![(old, new)],
        ..empty_changes()
    }];
    let msgs = collect_destructive_messages(&changes);
    assert!(
        msgs.iter().any(|m| m.contains("nullable -> not_null")),
        "{msgs:?}"
    );
}

#[test]
fn destructive_drop_fk_is_flagged() {
    let changes = vec![Changes {
        dropped_fks: vec![ParsedFk {
            from_column: "post_id".into(),
            to_table: "post".into(),
            to_column: "id".into(),
            on_delete: crate::migration::FkAction::NoAction,
            on_update: crate::migration::FkAction::NoAction,
        }],
        ..empty_changes()
    }];
    let msgs = collect_destructive_messages(&changes);
    assert!(
        msgs.iter().any(|m| m.contains("DROP FOREIGN KEY")),
        "{msgs:?}"
    );
}

#[test]
fn non_destructive_change_yields_no_messages() {
    // Adding a nullable column is safe.
    let changes = vec![Changes {
        added_columns: vec![ParsedColumn {
            name: "nickname".into(),
            col_type: crate::migration::ColumnType::String,
            nullable: true,
            ..Default::default()
        }],
        ..empty_changes()
    }];
    assert!(collect_destructive_messages(&changes).is_empty());
}

// A dropped table is a destructive element like the others, found by the
// same function, and listed first: a table created or altered is not one.
#[test]
fn a_dropped_table_is_listed_with_the_other_destructive_elements() {
    use crate::migration::TableChange;
    use crate::migration::steps::changes::{Destructive, DestructiveKind, destructive};

    let dropped_column = change(
        false,
        vec![],
        vec![ParsedColumn {
            name: "bio".into(),
            ..Default::default()
        }],
    );
    let dropped_table = Changes {
        table_name: "menus".into(),
        table: TableChange::Deleted,
        ..Default::default()
    };
    let created = Changes {
        table_name: "plats".into(),
        table: TableChange::Created,
        ..Default::default()
    };

    let found = destructive(&[dropped_column.clone(), created, dropped_table.clone()]);

    assert_eq!(
        found,
        [
            Destructive {
                table: "menus".into(),
                kind: DestructiveKind::DropTable,
            },
            Destructive {
                table: "t".into(),
                kind: DestructiveKind::DropColumn {
                    column: "bio".into()
                },
            },
        ]
    );
    assert_eq!(
        collect_destructive_messages(&[dropped_column, dropped_table]),
        [
            "  menus: DROP TABLE (data loss)",
            "  t.bio: DROP COLUMN (data loss)"
        ]
    );
    assert!(
        !Changes {
            table: TableChange::Deleted,
            ..empty_changes()
        }
        .is_empty()
    );
}
