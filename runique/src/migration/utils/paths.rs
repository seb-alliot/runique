/// Chemin du fichier CREATE principal (état courant de la table)
pub fn create_file_path(migrations_path: &str, table_name: &str) -> String {
    format!("{}/{}.rs", migrations_path, table_name)
}

/// Répertoire applied/ racine
pub fn applied_dir(migrations_path: &str) -> String {
    format!("{}/applied", migrations_path)
}

/// Répertoire applied/<table>/
pub fn table_applied_dir(migrations_path: &str, table_name: &str) -> String {
    format!("{}/applied/{}", migrations_path, table_name)
}

/// Chemin du fichier ALTER individuel
pub fn alter_file_path(migrations_path: &str, table_name: &str, timestamp: &str) -> String {
    format!(
        "{}/applied/{}/{}_alter_{}_table.rs",
        migrations_path, table_name, timestamp, table_name
    )
}

/// Répertoire applied/by_time/up/
pub fn batch_up_dir(migrations_path: &str) -> String {
    format!("{}/applied/by_time/up", migrations_path)
}

/// Répertoire applied/by_time/down/
pub fn batch_down_dir(migrations_path: &str) -> String {
    format!("{}/applied/by_time/down", migrations_path)
}

/// Chemin du fichier batch up agrégé
pub fn batch_up_path(migrations_path: &str, timestamp: &str) -> String {
    format!("{}/applied/by_time/up/{}.rs", migrations_path, timestamp)
}

/// Chemin du fichier batch down agrégé
pub fn batch_down_path(migrations_path: &str, timestamp: &str) -> String {
    format!("{}/applied/by_time/down/{}.rs", migrations_path, timestamp)
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

    // ── create_file_path ────────────────────────────────────────────────────

    #[test]
    fn create_file_path_standard() {
        assert_eq!(
            create_file_path(BASE, TABLE),
            "/project/migration/src/eihwaz_users.rs"
        );
    }

    #[test]
    fn create_file_path_simple_table() {
        assert_eq!(
            create_file_path("migrations", "posts"),
            "migrations/posts.rs"
        );
    }

    #[test]
    fn create_file_path_trailing_slash() {
        // Le caller ne doit pas passer de slash final — comportement documenté
        assert_eq!(
            create_file_path("migrations/", "posts"),
            "migrations//posts.rs"
        );
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
        // Le table_name apparaît dans le répertoire ET dans le nom de fichier
        assert!(path.contains("/applied/products/"));
        assert!(path.contains("_alter_products_table.rs"));
    }

    #[test]
    fn alter_file_path_timestamp_prefix() {
        let path = alter_file_path("m", "users", "20251231_235959");
        assert!(path.starts_with("m/applied/users/20251231_235959_alter_users_table.rs"));
    }

    // ── batch_up_path / batch_down_path ─────────────────────────────────────

    #[test]
    fn batch_up_path_format() {
        assert_eq!(
            batch_up_path(BASE, TS),
            "/project/migration/src/applied/by_time/up/20250218_143000.rs"
        );
    }

    #[test]
    fn batch_down_path_format() {
        assert_eq!(
            batch_down_path(BASE, TS),
            "/project/migration/src/applied/by_time/down/20250218_143000.rs"
        );
    }

    #[test]
    fn batch_up_and_down_share_timestamp() {
        let up = batch_up_path("m", TS);
        let down = batch_down_path("m", TS);
        // Même timestamp, seul up/down diffère
        assert!(up.contains("/by_time/up/"));
        assert!(down.contains("/by_time/down/"));
        assert!(up.ends_with(&format!("{}.rs", TS)));
        assert!(down.ends_with(&format!("{}.rs", TS)));
    }

    #[test]
    fn batch_up_and_down_differ_only_by_direction() {
        let up = batch_up_path("migrations", TS);
        let down = batch_down_path("migrations", TS);
        assert_ne!(up, down);
        assert_eq!(up.replace("/by_time/up/", "/by_time/down/"), down);
    }

    // ── batch_up_dir / batch_down_dir ────────────────────────────────────────

    #[test]
    fn batch_up_dir_format() {
        assert_eq!(batch_up_dir("migrations"), "migrations/applied/by_time/up");
    }

    #[test]
    fn batch_down_dir_format() {
        assert_eq!(
            batch_down_dir("migrations"),
            "migrations/applied/by_time/down"
        );
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
    fn alter_path_is_inside_table_applied_dir() {
        let dir = table_applied_dir(BASE, TABLE);
        let alter = alter_file_path(BASE, TABLE, TS);
        assert!(alter.starts_with(&dir));
    }

    #[test]
    fn batch_up_path_is_inside_batch_up_dir() {
        let dir = batch_up_dir(BASE);
        let path = batch_up_path(BASE, TS);
        assert!(path.starts_with(&dir));
    }

    #[test]
    fn batch_down_path_is_inside_batch_down_dir() {
        let dir = batch_down_dir(BASE);
        let path = batch_down_path(BASE, TS);
        assert!(path.starts_with(&dir));
    }

    #[test]
    fn applied_dir_is_prefix_of_table_applied_dir() {
        let base = applied_dir(BASE);
        let table = table_applied_dir(BASE, TABLE);
        assert!(table.starts_with(&base));
    }
}
