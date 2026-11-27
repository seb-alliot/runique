mod types_migration {
    use runique_cli::migration::types::*;
    #[test]
    fn test_parsed_schema() {
        let schema = ParsedSchema {
            table_name: "users".to_string(),
            primary_key: None,
            columns: vec![],
            foreign_keys: vec![],
            indexes: vec![],
        };
        assert_eq!(schema.table_name, "users");
    }

    #[test]
    fn test_parsed_column() {
        let col = ParsedColumn {
            name: "id".to_string(),
            col_type: runique_cli::migration::ColumnType::Integer,
            unique: true,
            ..ParsedColumn::default()
        };
        assert_eq!(col.name, "id");
        assert!(col.unique);
    }

    #[test]
    fn test_parsed_fk() {
        let fk = ParsedFk {
            from_column: "user_id".to_string(),
            to_table: "accounts".to_string(),
            to_column: "id".to_string(),
            on_delete: runique_cli::migration::FkAction::Cascade,
            on_update: runique_cli::migration::FkAction::NoAction,
        };
        assert_eq!(fk.from_column, "user_id");
        assert_eq!(fk.to_table, "accounts");
        assert_eq!(fk.to_column, "id");
        assert_eq!(fk.on_delete, runique_cli::migration::FkAction::Cascade);
    }

    #[test]
    fn test_parsed_index() {
        let idx = ParsedIndex {
            name: "idx".to_string(),
            columns: vec!["id".to_string()],
            unique: false,
        };
        assert_eq!(idx.columns, vec!["id"]);
    }

    #[test]
    fn test_changes() {
        let changes = Changes {
            table_name: String::new(),
            added_columns: vec![],
            dropped_columns: vec![],
            modified_columns: vec![],
            added_fks: vec![],
            dropped_fks: vec![],
            added_indexes: vec![],
            dropped_indexes: vec![],
            table: runique_cli::migration::TableChange::Altered,
            renames: Default::default(),
            renamed_columns: vec![],
        };
        assert!(changes.added_columns.is_empty());
    }
}
