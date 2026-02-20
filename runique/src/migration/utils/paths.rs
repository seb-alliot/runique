/// snapshots/ directory (current state of the table, used for diff only)
///
/// # Exemple
///
/// ```rust
/// use runique::migration::utils::paths::snapshot_dir;
/// assert_eq!(snapshot_dir("migrations"), "migrations/snapshots");
/// ```
pub fn snapshot_dir(migrations_path: &str) -> String {
    format!("{}/snapshots", migrations_path)
}

/// Path to the snapshot file of a table
///
/// # Exemple
///
/// ```rust
/// use runique::migration::utils::paths::snapshot_file_path;
/// assert_eq!(snapshot_file_path("migrations", "users"), "migrations/snapshots/users.rs");
/// ```
pub fn snapshot_file_path(migrations_path: &str, table_name: &str) -> String {
    format!("{}/snapshots/{}.rs", migrations_path, table_name)
}

/// SeaORM module name for a CREATE (used in lib.rs and as file name)
///
/// # Exemple
///
/// ```rust
/// use runique::migration::utils::paths::seaorm_create_module_name;
/// assert_eq!(seaorm_create_module_name("20260219", "blog"), "m20260219_create_blog_table");
/// ```
pub fn seaorm_create_module_name(timestamp: &str, table_name: &str) -> String {
    format!("m{}_create_{}_table", timestamp, table_name)
}

/// Path to the SeaORM migration file for a CREATE
///
/// # Exemple
///
/// ```rust
/// use runique::migration::utils::paths::seaorm_create_file_path;
/// assert_eq!(seaorm_create_file_path("migrations", "20260219", "blog"), "migrations/m20260219_create_blog_table.rs");
/// ```
pub fn seaorm_create_file_path(migrations_path: &str, timestamp: &str, table_name: &str) -> String {
    format!(
        "{}/m{}_create_{}_table.rs",
        migrations_path, timestamp, table_name
    )
}

/// Root applied/ directory
///
/// # Exemple
///
/// ```rust
/// use runique::migration::utils::paths::applied_dir;
/// assert_eq!(applied_dir("migrations"), "migrations/applied");
/// ```
pub fn applied_dir(migrations_path: &str) -> String {
    format!("{}/applied", migrations_path)
}

/// applied/<table>/ directory
///
/// # Exemple
///
/// ```rust
/// use runique::migration::utils::paths::table_applied_dir;
/// assert_eq!(table_applied_dir("migrations", "users"), "migrations/applied/users");
/// ```
pub fn table_applied_dir(migrations_path: &str, table_name: &str) -> String {
    format!("{}/applied/{}", migrations_path, table_name)
}

/// Path to the individual ALTER file
pub fn alter_file_path(migrations_path: &str, table_name: &str, timestamp: &str) -> String {
    format!(
        "{}/applied/{}/{}_alter_{}_table.rs",
        migrations_path, table_name, timestamp, table_name
    )
}

/// Root applied/by_time/ directory (for listing)
pub fn by_time_dir(migrations_path: &str) -> String {
    format!("{}/applied/by_time", migrations_path)
}

/// applied/by_time/<table>/ directory
pub fn by_time_table_dir(migrations_path: &str, table_name: &str) -> String {
    format!("{}/applied/by_time/{}", migrations_path, table_name)
}

/// applied/by_time/<table>/up/ directory
pub fn batch_up_dir(migrations_path: &str, table_name: &str) -> String {
    format!("{}/applied/by_time/{}/up", migrations_path, table_name)
}

/// applied/by_time/<table>/down/ directory
pub fn batch_down_dir(migrations_path: &str, table_name: &str) -> String {
    format!("{}/applied/by_time/{}/down", migrations_path, table_name)
}

/// Chemin du fichier batch up d'une table
pub fn batch_up_path(migrations_path: &str, table_name: &str, timestamp: &str) -> String {
    format!(
        "{}/applied/by_time/{}/up/{}.rs",
        migrations_path, table_name, timestamp
    )
}

/// Chemin du fichier batch down d'une table
pub fn batch_down_path(migrations_path: &str, table_name: &str, timestamp: &str) -> String {
    format!(
        "{}/applied/by_time/{}/down/{}.rs",
        migrations_path, table_name, timestamp
    )
}

/// Chemin du lib.rs du migrator
pub fn lib_path(migrations_path: &str) -> String {
    format!("{}/lib.rs", migrations_path)
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

    // ── applied_dir ─────────────────────────────────────────────────────────

    #[test]
    fn applied_dir_standard() {
        assert_eq!(applied_dir(BASE), "/project/migration/src/applied");
    }

    #[test]
    fn applied_dir_relative() {
        assert_eq!(applied_dir("migrations"), "migrations/applied");
    }

    // ── table_applied_dir ───────────────────────────────────────────────────

    #[test]
    fn table_applied_dir_standard() {
        assert_eq!(
            table_applied_dir(BASE, TABLE),
            "/project/migration/src/applied/eihwaz_users"
        );
    }

    #[test]
    fn table_applied_dir_different_table() {
        assert_eq!(
            table_applied_dir("migrations", "orders"),
            "migrations/applied/orders"
        );
    }

    // ── alter_file_path ─────────────────────────────────────────────────────

    #[test]
    fn alter_file_path_format() {
        assert_eq!(
            alter_file_path(BASE, TABLE, TS),
            "/project/migration/src/applied/eihwaz_users/20250218_143000_alter_eihwaz_users_table.rs"
        );
    }

    #[test]
    fn alter_file_path_embeds_table_name_twice() {
        let path = alter_file_path("m", "products", "20250101_120000");
        assert!(path.contains("/applied/products/"));
        assert!(path.contains("_alter_products_table.rs"));
    }

    #[test]
    fn alter_file_path_timestamp_prefix() {
        let path = alter_file_path("m", "users", "20251231_235959");
        assert!(path.starts_with("m/applied/users/20251231_235959_alter_users_table.rs"));
    }

    // ── by_time_dir / by_time_table_dir ─────────────────────────────────────

    #[test]
    fn by_time_dir_standard() {
        assert_eq!(by_time_dir(BASE), "/project/migration/src/applied/by_time");
    }

    #[test]
    fn by_time_table_dir_standard() {
        assert_eq!(
            by_time_table_dir(BASE, TABLE),
            "/project/migration/src/applied/by_time/eihwaz_users"
        );
    }

    // ── batch_up_dir / batch_down_dir ────────────────────────────────────────

    #[test]
    fn batch_up_dir_format() {
        assert_eq!(
            batch_up_dir("migrations", TABLE),
            "migrations/applied/by_time/eihwaz_users/up"
        );
    }

    #[test]
    fn batch_down_dir_format() {
        assert_eq!(
            batch_down_dir("migrations", TABLE),
            "migrations/applied/by_time/eihwaz_users/down"
        );
    }

    // ── batch_up_path / batch_down_path ─────────────────────────────────────

    #[test]
    fn batch_up_path_format() {
        assert_eq!(
            batch_up_path(BASE, TABLE, TS),
            "/project/migration/src/applied/by_time/eihwaz_users/up/20250218_143000.rs"
        );
    }

    #[test]
    fn batch_down_path_format() {
        assert_eq!(
            batch_down_path(BASE, TABLE, TS),
            "/project/migration/src/applied/by_time/eihwaz_users/down/20250218_143000.rs"
        );
    }

    #[test]
    fn batch_up_and_down_same_table_same_timestamp() {
        let up = batch_up_path("m", TABLE, TS);
        let down = batch_down_path("m", TABLE, TS);
        assert!(up.contains(&format!("/by_time/{}/up/", TABLE)));
        assert!(down.contains(&format!("/by_time/{}/down/", TABLE)));
        assert!(up.ends_with(&format!("{}.rs", TS)));
        assert!(down.ends_with(&format!("{}.rs", TS)));
    }

    #[test]
    fn batch_up_and_down_differ_only_by_direction() {
        let up = batch_up_path("migrations", TABLE, TS);
        let down = batch_down_path("migrations", TABLE, TS);
        assert_ne!(up, down);
        assert_eq!(
            up.replace(
                &format!("/by_time/{}/up/", TABLE),
                &format!("/by_time/{}/down/", TABLE)
            ),
            down
        );
    }

    #[test]
    fn different_tables_are_isolated() {
        let users_up = batch_up_path("m", "users", TS);
        let blog_up = batch_up_path("m", "blog", TS);
        assert_ne!(users_up, blog_up);
        assert!(users_up.contains("/by_time/users/up/"));
        assert!(blog_up.contains("/by_time/blog/up/"));
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

    // ── cohérence globale ────────────────────────────────────────────────────

    #[test]
    fn snapshot_is_inside_snapshot_dir() {
        let dir = snapshot_dir(BASE);
        let file = snapshot_file_path(BASE, TABLE);
        assert!(file.starts_with(&dir));
    }

    #[test]
    fn alter_path_is_inside_table_applied_dir() {
        let dir = table_applied_dir(BASE, TABLE);
        let alter = alter_file_path(BASE, TABLE, TS);
        assert!(alter.starts_with(&dir));
    }

    #[test]
    fn batch_up_path_is_inside_batch_up_dir() {
        let dir = batch_up_dir(BASE, TABLE);
        let path = batch_up_path(BASE, TABLE, TS);
        assert!(path.starts_with(&dir));
    }

    #[test]
    fn batch_down_path_is_inside_batch_down_dir() {
        let dir = batch_down_dir(BASE, TABLE);
        let path = batch_down_path(BASE, TABLE, TS);
        assert!(path.starts_with(&dir));
    }

    #[test]
    fn batch_table_dir_is_inside_by_time_dir() {
        let root = by_time_dir(BASE);
        let table = by_time_table_dir(BASE, TABLE);
        assert!(table.starts_with(&root));
    }

    #[test]
    fn applied_dir_is_prefix_of_table_applied_dir() {
        let base = applied_dir(BASE);
        let table = table_applied_dir(BASE, TABLE);
        assert!(table.starts_with(&base));
    }
}
