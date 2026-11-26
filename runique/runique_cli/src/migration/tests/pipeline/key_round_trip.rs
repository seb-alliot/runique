use crate::migration::tests::*;

// ── FK snapshot round-trip is stable ──

#[test]
fn fk_round_trip_is_stable() {
    let schema = ParsedSchema {
        table_name: "comment".into(),
        primary_key: Some(ParsedColumn {
            name: "id".into(),
            col_type: crate::migration::ColumnType::Integer,
            ..Default::default()
        }),
        columns: vec![ParsedColumn {
            name: "post_id".into(),
            col_type: crate::migration::ColumnType::Integer,
            ..Default::default()
        }],
        foreign_keys: vec![ParsedFk {
            from_column: "post_id".into(),
            to_table: "post".into(),
            to_column: "id".into(),
            on_delete: crate::migration::FkAction::Cascade,
            on_update: crate::migration::FkAction::NoAction,
        }],
        indexes: vec![],
    };
    let snapshot = generate_snapshot_file(&schema);
    let reparsed = parse_seaorm_source(&snapshot).expect("snapshot should reparse");
    let changes = diff_schemas(&reparsed, &schema);
    assert!(
        changes.added_fks.is_empty() && changes.dropped_fks.is_empty(),
        "FK round-trip not stable: {changes:?}"
    );
}

#[test]
fn fk_is_inlined_in_create_table_on_every_engine() {
    // SQLite cannot ALTER-ADD FKs, so they must be inlined in CREATE TABLE — and since
    // a migration file is fixed forever once committed, `generate_create_file` inlines
    // on every engine rather than deciding at generation time (same reasoning as the
    // enum/trigger runtime-guard fixes: no separate relations migration any more).
    let schema = ParsedSchema {
        table_name: "comment".into(),
        primary_key: Some(ParsedColumn {
            name: "id".into(),
            col_type: crate::migration::ColumnType::Integer,
            ..Default::default()
        }),
        columns: vec![ParsedColumn {
            name: "post_id".into(),
            col_type: crate::migration::ColumnType::Integer,
            ..Default::default()
        }],
        foreign_keys: vec![ParsedFk {
            from_column: "post_id".into(),
            to_table: "post".into(),
            to_column: "id".into(),
            on_delete: crate::migration::FkAction::Cascade,
            on_update: crate::migration::FkAction::NoAction,
        }],
        indexes: vec![],
    };

    let sqlite = generate_create_file(&schema);
    assert!(
        sqlite.contains(".foreign_key("),
        "SQLite must inline FK in CREATE:\n{sqlite}"
    );
    assert!(
        sqlite.contains(r#".from(Alias::new("comment"), Alias::new("post_id"))"#),
        "inline FK from clause:\n{sqlite}"
    );
    assert!(
        sqlite.contains(".on_delete(ForeignKeyAction::Cascade)"),
        "inline FK action:\n{sqlite}"
    );

    let pg = generate_create_file(&schema);
    assert!(
        pg.contains(".foreign_key("),
        "PG must also inline FK in CREATE:\n{pg}"
    );

    // The snapshot never inlines (keeps round-trip stable).
    let snap = generate_snapshot_file(&schema);
    assert!(
        !snap.contains(".foreign_key("),
        "snapshot must not inline FK:\n{snap}"
    );
}
