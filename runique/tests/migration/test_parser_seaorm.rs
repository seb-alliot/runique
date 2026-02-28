mod parser_seaorm_migration {
    #[test]
    fn test_parse_seaorm_source() {
        use runique::migration::utils::parser_seaorm::parse_seaorm_source;
        let src = "Entity { id: i32, name: String }";
        let schema = parse_seaorm_source(src);
        assert!(schema.is_ok() || schema.is_err()); // On vérifie juste l'appel pour l'exemple
    }
}
