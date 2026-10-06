mod paths_migration {
    use runique::migration::utils::paths::*;
    #[test]
    fn test_snapshot_dir() {
        assert!(snapshot_dir("test").contains("snapshot"));
    }

    #[test]
    fn test_snapshot_file_path() {
        let path = snapshot_file_path("test", "test");
        assert!(path.contains("test"));
    }

    #[test]
    fn test_seaorm_create_module_name() {
        let name = seaorm_create_module_name("users", "test");
        assert!(name.contains("users"));
    }

    #[test]
    fn test_seaorm_create_file_path() {
        let path = seaorm_create_file_path("users", "test", "test");
        assert!(path.contains("users"));
    }
}
