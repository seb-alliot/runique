mod convertisseur_migration {
    #[test]
    fn test_to_pascal_case() {
        use runique::migration::to_pascal_case;
        assert_eq!(to_pascal_case("test_case"), "TestCase");
        assert_eq!(to_pascal_case("foo_bar_baz"), "FooBarBaz");
        assert_eq!(to_pascal_case(""), "");
        assert_eq!(to_pascal_case("alreadyPascal"), "AlreadyPascal");
    }
}
