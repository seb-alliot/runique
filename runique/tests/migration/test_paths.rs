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

    #[test]
    fn test_applied_dir() {
        assert!(applied_dir("test").contains("applied"));
    }

    #[test]
    fn test_table_applied_dir() {
        let path = table_applied_dir("users", "test");
        assert!(path.contains("users"));
    }

    #[test]
    fn test_alter_file_path() {
        let path = alter_file_path("users", "test", "test");
        assert!(path.contains("users"));
    }

    #[test]
    fn test_by_time_dir() {
        assert!(by_time_dir("test").contains("by_time"));
    }
}
