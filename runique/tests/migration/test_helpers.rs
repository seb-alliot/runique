mod helpers_migration {
    // Tests pour col_type_to_method
    #[test]
    fn test_col_type_to_method_basic() {
        use runique::migration::col_type_to_method;
        // Cas non reconnu ("int") doit retourner "string()"
        assert_eq!(
            col_type_to_method("int"),
            "string()",
            "'int' doit retourner 'string()'"
        );
        // Cas exacts
        assert_eq!(
            col_type_to_method("Text"),
            "text()",
            "'Text' doit retourner 'text()'"
        );
        assert_eq!(
            col_type_to_method("TinyInteger"),
            "tiny_integer()",
            "'TinyInteger' doit retourner 'tiny_integer()'"
        );
        assert_eq!(
            col_type_to_method("Integer"),
            "integer()",
            "'Integer' doit retourner 'integer()'"
        );
        assert_eq!(
            col_type_to_method("Inconnu"),
            "string()",
            "'Inconnu' doit retourner 'string()'"
        );
    }

    #[test]
    fn test_detect_col_type_builder() {
        use runique::migration::detect_col_type_builder;
        let methods = vec!["tiny_integer".to_string()];
        assert_eq!(detect_col_type_builder(&methods), "TinyInteger");
        let methods = vec!["text".to_string()];
        assert_eq!(detect_col_type_builder(&methods), "Text");
        let methods = vec!["blob".to_string()];
        assert_eq!(detect_col_type_builder(&methods), "Blob");
        let methods = vec!["unknown".to_string()];
        assert_eq!(detect_col_type_builder(&methods), "String");
    }

    #[test]
    fn test_detect_col_type_seaorm() {
        use runique::migration::detect_col_type_seaorm;
        let methods = vec!["big_integer".to_string()];
        assert_eq!(detect_col_type_seaorm(&methods), "BigInteger");
        let methods = vec!["json".to_string()];
        assert_eq!(detect_col_type_seaorm(&methods), "Json");
        let methods = vec!["cidr".to_string()];
        assert_eq!(detect_col_type_seaorm(&methods), "Cidr");
        let methods = vec!["unknown".to_string()];
        assert_eq!(detect_col_type_seaorm(&methods), "String");
    }

    #[test]
    fn test_to_snake_case() {
        use runique::migration::to_snake_case;
        let result = to_snake_case("TestCase");
        assert_eq!(result, "test_case");
    }

    // Les tests AST nécessitent syn::parse_str, on vérifie juste que les helpers ne paniquent pas et retournent un résultat attendu minimal.
    use syn::{Expr, parse_str};

    #[test]
    fn test_collect_chain() {
        use runique::migration;
        let expr: Expr = parse_str("foo.bar().baz()").unwrap();
        let chain = migration::collect_chain(&expr);
        assert!(!chain.is_empty());
    }

    #[test]
    fn test_get_root_expr() {
        let expr: Expr = parse_str("foo.bar().baz()").unwrap();
        use runique::migration;
        let root = migration::get_root_expr(&expr);
        if let Expr::Path(_) = root {
            // ok
        } else {
            panic!("Root should be a path");
        }
    }

    #[test]
    fn test_first_str_arg() {
        let expr: Expr = parse_str("foo.bar(\"hello\")").unwrap();
        if let Expr::MethodCall(mc) = expr {
            use runique::migration;
            let arg = migration::first_str_arg(&mc);
            assert_eq!(arg, Some("hello".to_string()));
        }
    }

    #[test]
    fn test_method_names_in_expr() {
        let expr: Expr = parse_str("foo.bar().baz()").unwrap();
        use runique::migration;
        let names = migration::method_names_in_expr(&expr);
        assert!(names.contains(&"bar".to_string()));
        assert!(names.contains(&"baz".to_string()));
    }

    #[test]
    fn test_extract_str_from_call() {
        use runique::migration;
        let expr: Expr = parse_str("bar(\"hello\")").unwrap();
        let s = migration::extract_str_from_call(&expr);
        assert_eq!(s, Some("hello".to_string()));
    }

    #[test]
    fn test_extract_all_str_args() {
        use runique::migration;
        let expr: Expr = parse_str("foo.bar(\"a\", \"b\")").unwrap();
        let args = migration::extract_all_str_args(&expr);
        assert!(args.contains(&"a".to_string()));
        assert!(args.contains(&"b".to_string()));
    }

    #[test]
    fn test_extract_references_from_expr() {
        use runique::migration;
        let expr: Expr = parse_str("foo.references(\"table\")").unwrap();
        let refs = migration::extract_references_from_expr(&expr);
        assert_eq!(refs, Some(("table".to_string(), "id".to_string())));
    }

    #[test]
    fn test_extract_fk_action() {
        use runique::migration;
        let expr: Expr = parse_str("foo.on_delete(Cascade)").unwrap();
        let action = migration::extract_fk_action(&expr, "on_delete");
        assert_eq!(action, "Cascade");
    }

    #[test]
    fn test_extract_fk_action_value() {
        use runique::migration;
        let expr: Expr = parse_str("Cascade").unwrap();
        let action = migration::extract_fk_action_value(&expr);
        assert_eq!(action, "Cascade");
    }

    #[test]
    fn test_extract_alias_new_str() {
        use runique::migration;
        let expr: Expr = parse_str("Alias::new(\"foo\")").unwrap();
        let alias = migration::extract_alias_new_str(&expr);
        assert_eq!(alias, Some("foo".to_string()));
    }

    #[test]
    fn test_extract_alias_new_str_inner() {
        use runique::migration;
        let expr: Expr = parse_str("Alias::new(\"bar\")").unwrap();
        let alias = migration::extract_alias_new_str_inner(&expr);
        assert_eq!(alias, Some("bar".to_string()));
    }
}
