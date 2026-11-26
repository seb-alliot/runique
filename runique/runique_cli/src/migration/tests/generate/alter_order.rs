use crate::migration::tests::*;

// ── Combined ALTER ordering: rename runs before drop/add ──

#[test]
fn combined_alter_runs_rename_before_drop_and_add() {
    let changes = Changes {
        renames: Default::default(),
        renamed_columns: vec![("old_name".into(), "new_name".into())],
        added_columns: vec![ParsedColumn {
            name: "fresh".into(),
            col_type: crate::migration::ColumnType::Integer,
            nullable: true,
            ..Default::default()
        }],
        dropped_columns: vec![ParsedColumn {
            name: "gone".into(),
            ..Default::default()
        }],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let up = sql.split("async fn down").next().unwrap();
    let rename_pos = up.find("rename_column").expect("rename present");
    let drop_pos = up.find("drop_column").expect("drop present");
    let add_pos = up.find("add_column").expect("add present");
    assert!(
        rename_pos < drop_pos && rename_pos < add_pos,
        "RENAME must precede DROP/ADD in up:\n{sql}"
    );
}

// A dropped column is the one instruction of an ALTER that loses data: it
// closes `up`, after everything that can still be undone, and its way back
// opens `down`.
#[test]
fn a_dropped_column_closes_up_and_its_return_opens_down() {
    let changes = Changes {
        renamed_columns: vec![("old_name".into(), "new_name".into())],
        dropped_columns: vec![ParsedColumn {
            name: "gone".into(),
            col_type: crate::migration::ColumnType::Integer,
            nullable: true,
            ..Default::default()
        }],
        modified_columns: vec![(
            ParsedColumn {
                name: "kept".into(),
                col_type: crate::migration::ColumnType::String,
                nullable: true,
                ..Default::default()
            },
            ParsedColumn {
                name: "kept".into(),
                col_type: crate::migration::ColumnType::String,
                nullable: false,
                ..Default::default()
            },
        )],
        added_columns: vec![ParsedColumn {
            name: "fresh".into(),
            col_type: crate::migration::ColumnType::Integer,
            nullable: true,
            ..Default::default()
        }],
        added_fks: vec![fk("post_id", "post")],
        added_indexes: vec![ParsedIndex {
            name: "idx_t_fresh".into(),
            columns: vec!["fresh".into()],
            unique: false,
        }],
        enum_renames: vec![("status".into(), "Status".into(), "old".into(), "new".into())],
        enum_value_adds: vec![("status".into(), "Status".into(), "more".into())],
        ..empty_changes()
    };
    let sql = generate_alter_file(&changes);
    let (up, down) = sql.split_once("async fn down").unwrap();

    let dropped = up.find(r#".drop_column(Alias::new("gone"))"#).expect(up);
    for earlier in [
        "rename_column",
        "modify_column",
        "add_column",
        "add_foreign_key",
        "create_index",
        "rename_value",
        "add_value",
    ] {
        let position = up
            .rfind(earlier)
            .unwrap_or_else(|| panic!("{earlier}:\n{up}"));
        assert!(
            position < dropped,
            "{earlier} must come before the drop:\n{up}"
        );
    }

    let back = down
        .find(r#".add_column(ColumnDef::new(Alias::new("gone"))"#)
        .expect(down);
    for later in [
        "drop_index",
        "drop_foreign_key",
        "drop_column",
        "modify_column",
        "rename_value",
        "rename_column",
    ] {
        let position = down
            .find(later)
            .unwrap_or_else(|| panic!("{later}:\n{down}"));
        assert!(
            back < position,
            "the column must be back before {later}:\n{down}"
        );
    }
}

// Every action a key can carry is read back from a snapshot as it was
// written. `SetDefault` used to come back as `NoAction`: the key then looked
// changed at every `makemigrations`.
#[test]
fn every_key_action_is_read_back_from_a_snapshot() {
    use crate::migration::FkAction;
    for action in [
        FkAction::NoAction,
        FkAction::Cascade,
        FkAction::SetNull,
        FkAction::Restrict,
        FkAction::SetDefault,
    ] {
        let key = ParsedFk {
            on_delete: action,
            on_update: action,
            ..fk("post_id", "post")
        };
        let schema = ParsedSchema {
            table_name: "t".into(),
            foreign_keys: vec![key.clone()],
            ..Default::default()
        };

        let read = parse_seaorm_source(&generate_snapshot_file(&schema)).unwrap();

        assert_eq!(read.foreign_keys, [key], "{action:?}");
        assert!(diff_schemas(&read, &schema).is_empty(), "{action:?}");
    }
}
