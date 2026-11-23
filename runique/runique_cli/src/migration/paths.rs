//! Migration file paths — snapshots directory, migration files, SeaORM entities.

/// `snapshots/` directory (current table state, used only for diffing).
#[doc = include_str!("../../doc-tests/migration/paths_snapshot_dir.md")]
pub fn snapshot_dir(migrations_path: &str) -> String {
    format!("{}/snapshots", migrations_path)
}

/// Path to the snapshot file of a table
#[doc = include_str!("../../doc-tests/migration/paths_snapshot_file.md")]
pub fn snapshot_file_path(migrations_path: &str, table_name: &str) -> String {
    format!("{}/snapshots/{}.rs", migrations_path, table_name)
}

/// SeaORM module name for a CREATE (used in lib.rs and as file name)
#[doc = include_str!("../../doc-tests/migration/paths_seaorm_module.md")]
pub fn seaorm_create_module_name(timestamp: &str, table_name: &str) -> String {
    format!("m{}_create_{}_table", timestamp, table_name)
}

/// Path to the SeaORM migration file for a CREATE
#[doc = include_str!("../../doc-tests/migration/paths_seaorm_file.md")]
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

/// Builds the SeaORM module name for an ALTER migration on `table` (`m{timestamp}_alter_{table}_table`).
pub fn seaorm_alter_module_name(timestamp: &str, table: &str) -> String {
    format!("m{}_alter_{}_table", timestamp, table)
}

/// Builds the file path of the SeaORM ALTER migration for `table`.
pub fn seaorm_alter_file_path(migrations_path: &str, timestamp: &str, table: &str) -> String {
    format!(
        "{}/{}.rs",
        migrations_path,
        seaorm_alter_module_name(timestamp, table)
    )
}

/// Builds the SeaORM module name for an `extend!{}` migration on `table` (`m{timestamp}_extend_{table}_table`).
pub fn seaorm_extend_module_name(timestamp: &str, table: &str) -> String {
    format!("m{}_extend_{}_table", timestamp, table)
}

/// Builds the file path of the SeaORM `extend!{}` migration for `table`.
pub fn seaorm_extend_file_path(migrations_path: &str, timestamp: &str, table: &str) -> String {
    format!(
        "{}/m{}_extend_{}_table.rs",
        migrations_path, timestamp, table
    )
}

/// SeaORM module name for the migration that drops `table`
/// (`m{timestamp}_drop_{table}_table`).
pub fn seaorm_drop_module_name(timestamp: &str, table: &str) -> String {
    format!("m{}_drop_{}_table", timestamp, table)
}

/// Builds the file path of the SeaORM migration that drops `table`.
pub fn seaorm_drop_file_path(migrations_path: &str, timestamp: &str, table: &str) -> String {
    format!(
        "{}/{}.rs",
        migrations_path,
        seaorm_drop_module_name(timestamp, table)
    )
}

/// What follows the date and the time in a generated module name:
/// `{kind}_{table}`, without the closing `_table`.
fn kind_and_table(module: &str) -> Option<&str> {
    // `m{date}_{time}_{kind}_{table}_table`
    module
        .strip_prefix('m')?
        .splitn(3, '_')
        .nth(2)?
        .strip_suffix("_table")
}

/// The table a generated migration module changes, and whether it is an
/// `extend!{}` one. `None` for a module Runique didn't name.
pub fn migration_target(module: &str) -> Option<(bool, &str)> {
    let rest = kind_and_table(module)?;
    if let Some(table) = rest.strip_prefix("extend_") {
        return Some((true, table));
    }
    rest.strip_prefix("create_")
        .or_else(|| rest.strip_prefix("alter_"))
        .or_else(|| rest.strip_prefix("drop_"))
        .map(|table| (false, table))
}

/// The table a generated migration module drops; `None` for any other module.
pub fn dropped_table(module: &str) -> Option<&str> {
    kind_and_table(module)?.strip_prefix("drop_")
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

    // ── migration_target ─────────────────────────────────────────────────────

    // The module names are read back to the table they were built from.
    #[test]
    fn a_generated_module_names_its_table() {
        assert_eq!(
            migration_target(&seaorm_create_module_name(TS, "menus")),
            Some((false, "menus"))
        );
        assert_eq!(
            migration_target(&seaorm_alter_module_name(TS, "menu_du_jour")),
            Some((false, "menu_du_jour"))
        );
        assert_eq!(
            migration_target(&seaorm_extend_module_name(TS, TABLE)),
            Some((true, TABLE))
        );
        // The kind is read first: a table may be named after another kind.
        assert_eq!(
            migration_target(&seaorm_create_module_name(TS, "alter_log")),
            Some((false, "alter_log"))
        );
    }

    #[test]
    fn a_module_runique_did_not_name_has_no_target() {
        for module in [
            "m20250218_143000_seed_menus_table",
            "m20250218_143000_create_menus",
            "20250218_143000_create_menus_table",
            "m20250218_create_menus_table",
            "create_menus_table",
            "",
        ] {
            assert_eq!(migration_target(module), None, "{module}");
        }
    }

    // ── a migration that drops a table ───────────────────────────────────────

    #[test]
    fn a_drop_module_names_its_table() {
        let module = seaorm_drop_module_name(TS, "menu_du_jour");
        assert_eq!(module, "m20250218_143000_drop_menu_du_jour_table");
        assert!(
            seaorm_drop_file_path(BASE, TS, "menu_du_jour").ends_with(&format!("/{module}.rs"))
        );
        // Same table as its creation: the two follow each other in a chain.
        assert_eq!(migration_target(&module), Some((false, "menu_du_jour")));
        assert_eq!(dropped_table(&module), Some("menu_du_jour"));
    }

    #[test]
    fn only_a_drop_module_drops_a_table() {
        for module in [
            seaorm_create_module_name(TS, "menus"),
            seaorm_alter_module_name(TS, "menus"),
            seaorm_extend_module_name(TS, TABLE),
            // A table named after the kind is still created, not dropped.
            seaorm_create_module_name(TS, "drop_log"),
            "m20250218_143000_drop_menus".to_string(),
            "drop_menus_table".to_string(),
        ] {
            assert_eq!(dropped_table(&module), None, "{module}");
        }
    }
}
