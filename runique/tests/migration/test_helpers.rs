mod helpers_migration {
    // A type gives the method a migration calls for it.
    #[test]
    fn test_column_type_method_basic() {
        use runique_cli::migration::ColumnType;
        assert_eq!(ColumnType::Text.method(), "text()");
        assert_eq!(ColumnType::TinyInteger.method(), "tiny_integer()");
        assert_eq!(ColumnType::Integer.method(), "integer()");
        assert_eq!(ColumnType::String.method(), "string()");
    }

    // A name that is not a type has no type: there is no fallback any more.
    #[test]
    fn test_column_type_from_name() {
        use runique_cli::migration::ColumnType;
        assert_eq!(ColumnType::from_name("Integer"), Some(ColumnType::Integer));
        assert_eq!(ColumnType::from_name("int"), None);
        assert_eq!(ColumnType::from_name("Inconnu"), None);
        assert_eq!(ColumnType::from_name(""), None);
    }

    #[test]
    fn test_column_type_from_methods() {
        use runique_cli::migration::ColumnType;
        let methods = vec!["big_integer".to_string()];
        assert_eq!(ColumnType::from_methods(&methods), "BigInteger");
        let methods = vec!["json".to_string()];
        assert_eq!(ColumnType::from_methods(&methods), "Json");
        let methods = vec!["cidr".to_string()];
        assert_eq!(ColumnType::from_methods(&methods), "Cidr");
        // No type method: the column is a string (an enum column has none).
        let methods = vec!["not_null".to_string()];
        assert_eq!(ColumnType::from_methods(&methods), "String");
    }

    // Les tests AST nécessitent syn::parse_str, on vérifie juste que les helpers ne paniquent pas et retournent un résultat attendu minimal.
    use syn::{Expr, parse_str};

    #[test]
    fn test_collect_chain() {
        use runique_cli::migration;
        let expr: Expr = parse_str("foo.bar().baz()").unwrap();
        let chain = migration::collect_chain(&expr);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_get_root_expr() {
        let expr: Expr = parse_str("foo.bar().baz()").unwrap();
        use runique_cli::migration;
        let root = migration::get_root_expr(&expr);
        if let Expr::Path(_) = root {
            // ok
        } else {
            panic!("Root should be a path");
        }
    }

    #[test]
    fn test_method_names_in_expr() {
        let expr: Expr = parse_str("foo.bar().baz()").unwrap();
        use runique_cli::migration;
        let names = migration::method_names_in_expr(&expr);
        assert!(names.contains(&"bar".to_string()));
        assert!(names.contains(&"baz".to_string()));
    }

    #[test]
    fn test_extract_str_from_call() {
        use runique_cli::migration;
        let expr: Expr = parse_str("bar(\"hello\")").unwrap();
        let s = migration::extract_str_from_call(&expr);
        assert_eq!(s, Some("hello".to_string()));
    }

    #[test]
    fn test_extract_all_str_args() {
        use runique_cli::migration;
        let expr: Expr = parse_str("foo.bar(\"a\", \"b\")").unwrap();
        let args = migration::extract_all_str_args(&expr);
        assert!(args.contains(&"a".to_string()));
        assert!(args.contains(&"b".to_string()));
    }

    #[test]
    fn test_extract_references_from_expr() {
        use runique_cli::migration;
        let expr: Expr = parse_str("foo.references(\"table\")").unwrap();
        let refs = migration::extract_references_from_expr(&expr);
        assert_eq!(refs, Some(("table".to_string(), "id".to_string())));
    }

    #[test]
    fn test_extract_fk_action() {
        use runique_cli::migration;
        let expr: Expr = parse_str("foo.on_delete(Cascade)").unwrap();
        let action = migration::extract_fk_action(&expr, "on_delete");
        assert_eq!(action, runique_cli::migration::FkAction::Cascade);
    }

    #[test]
    fn test_extract_fk_action_value() {
        use runique_cli::migration;
        let expr: Expr = parse_str("Cascade").unwrap();
        let action = migration::extract_fk_action_value(&expr);
        assert_eq!(action, runique_cli::migration::FkAction::Cascade);
    }

    #[test]
    fn test_extract_alias_new_str() {
        use runique_cli::migration;
        let expr: Expr = parse_str("Alias::new(\"foo\")").unwrap();
        let alias = migration::extract_alias_new_str(&expr);
        assert_eq!(alias, Some("foo".to_string()));
    }

    #[test]
    fn test_extract_alias_new_str_inner() {
        use runique_cli::migration;
        let expr: Expr = parse_str("Alias::new(\"bar\")").unwrap();
        let alias = migration::extract_alias_new_str_inner(&expr);
        assert_eq!(alias, Some("bar".to_string()));
    }
}

// Written from cargo-mutants survivors (2026-10-07): every type, and every
// alternative spelling a detector accepts, checked one by one.
mod helpers_type_table {
    use runique_cli::migration::ColumnType;

    const TYPES: &[(&str, &str)] = &[
        ("String", "string"),
        ("Text", "text"),
        ("TinyInteger", "tiny_integer"),
        ("SmallInteger", "small_integer"),
        ("Integer", "integer"),
        ("BigInteger", "big_integer"),
        ("Unsigned", "unsigned"),
        ("BigUnsigned", "big_unsigned"),
        ("Float", "float"),
        ("Double", "double"),
        ("Decimal", "decimal"),
        ("Boolean", "boolean"),
        ("DateTime", "date_time"),
        ("Timestamp", "timestamp"),
        ("TimestampWithTimeZone", "timestamp_with_time_zone"),
        ("Date", "date"),
        ("Time", "time"),
        ("Uuid", "uuid"),
        ("Json", "json"),
        ("JsonBinary", "json_binary"),
        ("Binary", "binary"),
        ("VarBinary", "var_binary"),
        ("Blob", "blob"),
        ("Char", "char"),
        ("Inet", "inet"),
        ("Cidr", "cidr"),
        ("MacAddr", "mac_address"),
        ("Interval", "interval"),
        ("Enum", "enum_type"),
    ];

    fn m(names: &[&str]) -> Vec<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    /// Every type of the list is in the table above, and the table holds
    /// nothing else: a type added to the list must be added here.
    #[test]
    fn the_table_holds_every_type_once() {
        assert_eq!(TYPES.len(), ColumnType::ALL.len());
        for column_type in ColumnType::ALL {
            let rows = TYPES
                .iter()
                .filter(|(name, _)| *column_type == *name)
                .count();
            assert_eq!(rows, 1, "{column_type}");
        }
    }

    /// What `makemigrations` writes for a type is read back as that same
    /// type, and its name gives it back too.
    #[test]
    fn every_type_round_trips_through_its_method() {
        for (name, method) in TYPES {
            let column_type = ColumnType::from_name(name).expect(name);
            assert_eq!(column_type.as_str(), *name);
            assert_eq!(column_type.to_string(), *name);
            // Three types have no call without an argument in sea-query.
            let written = match *name {
                "VarBinary" => "var_binary(255)".to_string(),
                "Interval" => "interval(None, None)".to_string(),
                "Enum" => "string()".to_string(),
                _ => format!("{method}()"),
            };
            assert_eq!(column_type.method(), written, "{name}");
            assert_eq!(
                ColumnType::from_methods(&m(&[method])),
                column_type,
                "{method}"
            );
        }
    }

    #[test]
    fn every_alternative_spelling_is_read() {
        for (methods, name) in [
            (&["binary_len"][..], "Binary"),
            (&["char_len"], "Char"),
            (&["decimal_len"], "Decimal"),
            (&["timestamp_tz"], "TimestampWithTimeZone"),
            (&["auto_now"], "DateTime"),
            (&["auto_now_update"], "DateTime"),
            (&["enumeration"], "Enum"),
            (&["string"], "String"),
            (&["string_len"], "String"),
        ] {
            assert_eq!(ColumnType::from_methods(&m(methods)), name, "{methods:?}");
        }
    }

    // The more specific method wins when a definition calls several.
    #[test]
    fn the_first_type_of_the_list_wins() {
        assert_eq!(ColumnType::from_methods(&m(&["blob", "text"])), "Blob");
        assert_eq!(ColumnType::from_methods(&m(&["text", "integer"])), "Text");
        assert_eq!(
            ColumnType::from_methods(&m(&["json_binary", "json"])),
            "JsonBinary"
        );
        assert_eq!(
            ColumnType::from_methods(&m(&["date_time", "date", "time"])),
            "DateTime"
        );
    }
}
