//! Migration file paths — snapshots directory, migration files, SeaORM entities.

/// `snapshots/` directory (current table state, used only for diffing).
#[doc = include_str!("../../../doc-tests/migration/paths_snapshot_dir.md")]
pub fn snapshot_dir(migrations_path: &str) -> String {
    format!("{}/snapshots", migrations_path)
}

/// Path to the snapshot file of a table
#[doc = include_str!("../../../doc-tests/migration/paths_snapshot_file.md")]
pub fn snapshot_file_path(migrations_path: &str, table_name: &str) -> String {
    format!("{}/snapshots/{}.rs", migrations_path, table_name)
}

/// SeaORM module name for a CREATE (used in lib.rs and as file name)
#[doc = include_str!("../../../doc-tests/migration/paths_seaorm_module.md")]
pub fn seaorm_create_module_name(timestamp: &str, table_name: &str) -> String {
    format!("m{}_create_{}_table", timestamp, table_name)
}

/// Path to the SeaORM migration file for a CREATE
#[doc = include_str!("../../../doc-tests/migration/paths_seaorm_file.md")]
pub fn seaorm_create_file_path(migrations_path: &str, timestamp: &str, table_name: &str) -> String {
    format!(
        "{}/m{}_create_{}_table.rs",
        migrations_path, timestamp, table_name
    )
}

/// Path to the migrator's lib.rs
pub fn lib_path(migrations_path: &str) -> String {
    format!("{}/lib.rs", migrations_path)
}

/// Directory for framework table extension snapshots (`snapshots/runique/`)
pub fn extend_snapshot_dir(migrations_path: &str) -> String {
    format!("{}/snapshots/runique", migrations_path)
}

/// Path to the extension snapshot for a given framework table
pub fn extend_snapshot_file_path(migrations_path: &str, table_name: &str) -> String {
    format!("{}/snapshots/runique/{}.rs", migrations_path, table_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "/project/migration/src";
    const TABLE: &str = "eihwaz_users";
    const TS: &str = "20250218_143000";

    // ── snapshot_dir ────────────────────────────────────────────────────────

    #[test]
    fn snapshot_dir_standard() {
        assert_eq!(snapshot_dir(BASE), "/project/migration/src/snapshots");
    }

    #[test]
    fn snapshot_dir_relative() {
        assert_eq!(snapshot_dir("migrations"), "migrations/snapshots");
    }

    // ── snapshot_file_path ──────────────────────────────────────────────────

    #[test]
    fn snapshot_file_path_standard() {
        assert_eq!(
            snapshot_file_path(BASE, TABLE),
            "/project/migration/src/snapshots/eihwaz_users.rs"
        );
    }

    #[test]
    fn snapshot_file_path_simple_table() {
        assert_eq!(
            snapshot_file_path("migrations", "posts"),
            "migrations/snapshots/posts.rs"
        );
    }

    // ── seaorm_create_module_name ────────────────────────────────────────────

    #[test]
    fn seaorm_create_module_name_format() {
        assert_eq!(
            seaorm_create_module_name(TS, TABLE),
            "m20250218_143000_create_eihwaz_users_table"
        );
    }

    #[test]
    fn seaorm_create_module_name_simple() {
        assert_eq!(
            seaorm_create_module_name("20260118_003649", "users"),
            "m20260118_003649_create_users_table"
        );
    }

    // ── seaorm_create_file_path ──────────────────────────────────────────────

    #[test]
    fn seaorm_create_file_path_standard() {
        assert_eq!(
            seaorm_create_file_path(BASE, TS, TABLE),
            "/project/migration/src/m20250218_143000_create_eihwaz_users_table.rs"
        );
    }

    #[test]
    fn seaorm_create_file_path_simple() {
        assert_eq!(
            seaorm_create_file_path("migrations", "20260118_003649", "users"),
            "migrations/m20260118_003649_create_users_table.rs"
        );
    }

    #[test]
    fn seaorm_create_module_name_matches_file_stem() {
        let module = seaorm_create_module_name(TS, TABLE);
        let file = seaorm_create_file_path(BASE, TS, TABLE);
        assert!(file.ends_with(&format!("{}.rs", module)));
    }

    // ── lib_path ─────────────────────────────────────────────────────────────

    #[test]
    fn lib_path_standard() {
        assert_eq!(lib_path(BASE), "/project/migration/src/lib.rs");
    }

    #[test]
    fn lib_path_relative() {
        assert_eq!(lib_path("migrations"), "migrations/lib.rs");
    }

    // ── global coherence ────────────────────────────────────────────────────

    #[test]
    fn snapshot_is_inside_snapshot_dir() {
        let dir = snapshot_dir(BASE);
        let file = snapshot_file_path(BASE, TABLE);
        assert!(file.starts_with(&dir));
    }
}
