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

/// Directory of the enum snapshots (`snapshots/enums/`): one file per enum,
/// beside the snapshots of the tables.
pub fn enum_snapshot_dir(migrations_path: &str) -> String {
    format!("{}/snapshots/enums", migrations_path)
}

/// Path to the snapshot of an enum, by the name the database gives it.
pub fn enum_snapshot_file_path(migrations_path: &str, enum_name: &str) -> String {
    format!("{}/snapshots/enums/{}.rs", migrations_path, enum_name)
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

/// SeaORM module name for the migration that creates the enum `name`
/// (`m{timestamp}_create_{name}_enum`).
pub fn seaorm_enum_create_module_name(timestamp: &str, name: &str) -> String {
    format!("m{}_create_{}_enum", timestamp, name)
}

/// SeaORM module name for the migration that gives the enum the name `name`
/// (`m{timestamp}_rename_{name}_enum`).
pub fn seaorm_enum_rename_module_name(timestamp: &str, name: &str) -> String {
    format!("m{}_rename_{}_enum", timestamp, name)
}

/// SeaORM module name for the migration that changes the values of the enum
/// `name` (`m{timestamp}_alter_{name}_enum`).
pub fn seaorm_enum_alter_module_name(timestamp: &str, name: &str) -> String {
    format!("m{}_alter_{}_enum", timestamp, name)
}

/// Builds the file path of a migration of an enum from its module name.
pub fn seaorm_enum_file_path(migrations_path: &str, module: &str) -> String {
    format!("{}/{}.rs", migrations_path, module)
}

/// What follows the date and the time in a generated module name:
/// `{kind}_{table}`, without the closing `_table`.
fn action_and_table(module: &str) -> Option<&str> {
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
    let rest = action_and_table(module)?;
    if let Some(table) = rest.strip_prefix("extend_") {
        return Some((true, table));
    }
    rest.strip_prefix("create_")
        .or_else(|| rest.strip_prefix("alter_"))
        .or_else(|| rest.strip_prefix("drop_"))
        .map(|table| (false, table))
}

/// The enum a generated migration module creates, renames or changes, by the
/// name the database gives it once the migration is applied. `None` for any other module: a module of a table
/// ends on `_table`, never on `_enum`.
pub fn enum_target(module: &str) -> Option<&str> {
    // `m{date}_{time}_{kind}_{enum}_enum`
    let rest = module
        .strip_prefix('m')?
        .splitn(3, '_')
        .nth(2)?
        .strip_suffix("_enum")?;
    rest.strip_prefix("create_")
        .or_else(|| rest.strip_prefix("rename_"))
        .or_else(|| rest.strip_prefix("alter_"))
        .filter(|name| !name.is_empty())
}

/// What a generated migration module is about: a table of a model, a table
/// an `extend!{}` block adds to, or an enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleTarget<'a> {
    Table(&'a str),
    Extend(&'a str),
    Enum(&'a str),
}

impl ModuleTarget<'_> {
    /// The name of the table or of the enum.
    pub fn name(&self) -> &str {
        match self {
            ModuleTarget::Table(name) | ModuleTarget::Extend(name) | ModuleTarget::Enum(name) => {
                name
            }
        }
    }
}

/// What a generated migration module is about; `None` for a module Runique
/// didn't name. Two modules about the same thing follow each other.
pub fn module_target(module: &str) -> Option<ModuleTarget<'_>> {
    match migration_target(module) {
        Some((true, table)) => Some(ModuleTarget::Extend(table)),
        Some((false, table)) => Some(ModuleTarget::Table(table)),
        None => enum_target(module).map(ModuleTarget::Enum),
    }
}

/// The table a generated migration module drops; `None` for any other module.
pub fn dropped_table(module: &str) -> Option<&str> {
    action_and_table(module)?.strip_prefix("drop_")
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

    // ── enum snapshots ──────────────────────────────────────────────────────

    #[test]
    fn an_enum_snapshot_is_in_its_own_folder_of_the_snapshots() {
        assert_eq!(
            enum_snapshot_dir(BASE),
            "/project/migration/src/snapshots/enums"
        );
        assert_eq!(
            enum_snapshot_file_path(BASE, "genre"),
            "/project/migration/src/snapshots/enums/genre.rs"
        );
        assert!(enum_snapshot_file_path(BASE, "genre").starts_with(&enum_snapshot_dir(BASE)));
        assert!(enum_snapshot_dir(BASE).starts_with(&snapshot_dir(BASE)));
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

    // ── a migration of an enum ───────────────────────────────────────────────

    #[test]
    fn a_module_of_an_enum_names_its_enum() {
        let created = seaorm_enum_create_module_name(TS, "genre");
        let altered = seaorm_enum_alter_module_name(TS, "order_status");
        assert_eq!(created, "m20250218_143000_create_genre_enum");
        assert_eq!(altered, "m20250218_143000_alter_order_status_enum");
        assert_eq!(enum_target(&created), Some("genre"));
        assert_eq!(enum_target(&altered), Some("order_status"));
        // A renamed enum is named as it is once renamed.
        let renamed = seaorm_enum_rename_module_name(TS, "category");
        assert_eq!(renamed, "m20250218_143000_rename_category_enum");
        assert_eq!(enum_target(&renamed), Some("category"));
        assert_eq!(
            seaorm_enum_file_path(BASE, &created),
            "/project/migration/src/m20250218_143000_create_genre_enum.rs"
        );
        // An enum is not a table, and a table is not an enum.
        assert_eq!(migration_target(&created), None);
        assert_eq!(dropped_table(&created), None);
        assert_eq!(enum_target(&seaorm_create_module_name(TS, "genre")), None);
        // A table named like an enum module ends on `_table` all the same.
        assert_eq!(
            enum_target(&seaorm_create_module_name(TS, "genre_enum")),
            None
        );
    }

    #[test]
    fn a_module_runique_did_not_name_is_no_enum_module() {
        for module in [
            "m20250218_143000_seed_genre_enum",
            "m20250218_143000_drop_genre_enum",
            "m20250218_143000_create_genre",
            "m20250218_143000_create__enum",
            "20250218_143000_create_genre_enum",
            "m20250218_create_genre_enum",
            "create_genre_enum",
            "",
        ] {
            assert_eq!(enum_target(module), None, "{module}");
        }
    }

    // One question for the three kinds of module, and none answers for
    // another: a table, an extension and an enum of the same name differ.
    #[test]
    fn what_a_module_is_about_tells_tables_extensions_and_enums_apart() {
        let modules = [
            seaorm_alter_module_name(TS, "genre"),
            seaorm_extend_module_name(TS, "genre"),
            seaorm_enum_alter_module_name(TS, "genre"),
        ];
        let table = module_target(&modules[0]).unwrap();
        let extend = module_target(&modules[1]).unwrap();
        let of_enum = module_target(&modules[2]).unwrap();

        assert_eq!(table, ModuleTarget::Table("genre"));
        assert_eq!(extend, ModuleTarget::Extend("genre"));
        assert_eq!(of_enum, ModuleTarget::Enum("genre"));
        assert!(table != extend && extend != of_enum && table != of_enum);
        for target in [table, extend, of_enum] {
            assert_eq!(target.name(), "genre");
        }
        // Created then changed: the same enum, the two follow each other.
        assert_eq!(
            module_target(&seaorm_enum_create_module_name(TS, "genre")),
            Some(of_enum)
        );
        assert_eq!(module_target("m20250218_143000_seed_genre_table"), None);
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
