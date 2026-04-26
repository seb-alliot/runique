mod types_migration {
    use runique::migration::utils::types::*;
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
            col_type: "Integer".to_string(),
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
            on_delete: "Cascade".to_string(),
            on_update: String::new(),
        };
        assert_eq!(fk.from_column, "user_id");
        assert_eq!(fk.to_table, "accounts");
        assert_eq!(fk.to_column, "id");
        assert_eq!(fk.on_delete, "Cascade".to_string());
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
            is_new_table: false,
            enum_renames: vec![],
            enum_value_adds: vec![],
            enum_value_drops: vec![],
        };
        assert!(changes.added_columns.is_empty());
    }
}
