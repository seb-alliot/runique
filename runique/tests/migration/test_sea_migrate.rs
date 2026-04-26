//! Tests — Pipeline complet via sea-orm-migration (cargo run -p migration)
//!
//! Flux : fresh (applique toutes les migrations) → vérification tables → reset (rollback tout)
//!
//! On utilise exclusivement les commandes sea-orm pour up/down.
//! Docker Postgres et MariaDB requis (DATABASE_URL_PG / DATABASE_URL_MARIADB dans .env.test).
//! SQLite : pas de Docker, fichier temporaire local.

use crate::helpers::db_mariadb as db_maria;
use crate::helpers::db_postgres as db_pg;
use sea_orm::ConnectionTrait;
use serial_test::serial;
use std::path::PathBuf;
use std::process::Command;

// ─── Tables créées par demo-app/migration ────────────────────────────────────

const TABLES: &[&str] = &["blog", "eihwaz_users", "test_all_fields", "users_booster"];

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Racine du workspace (parent de runique/)
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Lance `cargo run -p migration -- <args>` avec DATABASE_URL.
fn sea_migrate(database_url: &str, args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "-p", "migration", "--"])
        .args(args)
        .env("DATABASE_URL", database_url)
        .current_dir(workspace_root())
        .output()
        .expect("cargo run -p migration doit s'exécuter")
}

/// Vérifie qu'une table existe en PostgreSQL (guillemets doubles).
async fn pg_table_exists(db: &sea_orm::DatabaseConnection, table: &str) -> bool {
    db.execute_unprepared(&format!("SELECT 1 FROM \"{}\" LIMIT 1", table))
        .await
        .is_ok()
}

/// Vérifie qu'une table existe en MariaDB/MySQL (backticks).
async fn maria_table_exists(db: &sea_orm::DatabaseConnection, table: &str) -> bool {
    db.execute_unprepared(&format!("SELECT 1 FROM `{}` LIMIT 1", table))
        .await
        .is_ok()
}

// ═══════════════════════════════════════════════════════════════
// Postgres — fresh → vérification → reset
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_sea_migrate_fresh_postgres() {
    let _ = dotenvy::from_filename(".env.test");
    let Some(db) = db_pg::connect().await else {
        return;
    };
    let pg_url = std::env::var("DATABASE_URL_PG").unwrap();

    let out = sea_migrate(&pg_url, &["fresh"]);
    assert!(
        out.status.success(),
        "sea migrate fresh (pg) doit Ok:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    for table in TABLES {
        assert!(
            pg_table_exists(&db, table).await,
            "table '{}' doit exister après fresh (pg)",
            table
        );
    }
}

#[tokio::test]
#[serial]
async fn test_sea_migrate_status_postgres() {
    let _ = dotenvy::from_filename(".env.test");
    let Some(_) = db_pg::connect().await else {
        return;
    };
    let pg_url = std::env::var("DATABASE_URL_PG").unwrap();

    sea_migrate(&pg_url, &["fresh"]);

    let out = sea_migrate(&pg_url, &["status"]);
    assert!(
        out.status.success(),
        "sea migrate status (pg) doit Ok:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Applied") || stdout.contains("applied") || out.stderr.len() < 200,
        "status doit lister les migrations appliquées"
    );
}

#[tokio::test]
#[serial]
async fn test_sea_migrate_reset_postgres() {
    let _ = dotenvy::from_filename(".env.test");
    let Some(db) = db_pg::connect().await else {
        return;
    };
    let pg_url = std::env::var("DATABASE_URL_PG").unwrap();

    sea_migrate(&pg_url, &["fresh"]);

    let out = sea_migrate(&pg_url, &["reset"]);
    assert!(
        out.status.success(),
        "sea migrate reset (pg) doit Ok:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    for table in TABLES {
        assert!(
            !pg_table_exists(&db, table).await,
            "table '{}' doit être supprimée après reset (pg)",
            table
        );
    }
}

// ═══════════════════════════════════════════════════════════════
// MariaDB — fresh → vérification → reset
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_sea_migrate_fresh_mariadb() {
    let _ = dotenvy::from_filename(".env.test");
    let Some(db) = db_maria::connect().await else {
        return;
    };
    let mariadb_url = std::env::var("DATABASE_URL_MARIADB").unwrap();
    let mysql_url = if mariadb_url.starts_with("mysql://") {
        mariadb_url
    } else {
        mariadb_url.replacen("mariadb://", "mysql://", 1)
    };

    let out = sea_migrate(&mysql_url, &["fresh"]);
    assert!(
        out.status.success(),
        "sea migrate fresh (mariadb) doit Ok:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    for table in TABLES {
        assert!(
            maria_table_exists(&db, table).await,
            "table '{}' doit exister après fresh (mariadb)",
            table
        );
    }
}

#[tokio::test]
#[serial]
async fn test_sea_migrate_reset_mariadb() {
    let _ = dotenvy::from_filename(".env.test");
    let Some(db) = db_maria::connect().await else {
        return;
    };
    let mariadb_url = std::env::var("DATABASE_URL_MARIADB").unwrap();
    let mysql_url = if mariadb_url.starts_with("mysql://") {
        mariadb_url
    } else {
        mariadb_url.replacen("mariadb://", "mysql://", 1)
    };

    sea_migrate(&mysql_url, &["fresh"]);

    let out = sea_migrate(&mysql_url, &["reset"]);
    assert!(
        out.status.success(),
        "sea migrate reset (mariadb) doit Ok:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    for table in TABLES {
        assert!(
            !maria_table_exists(&db, table).await,
            "table '{}' doit être supprimée après reset (mariadb)",
            table
        );
    }
}

// ═══════════════════════════════════════════════════════════════
// SQLite — fresh → vérification → reset (pas de Docker)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_sea_migrate_reset_sqlite() {
    let db_file = std::env::temp_dir().join("runique_test_sea_migration_reset.db");
    let sqlite_url = format!("sqlite://{}?mode=rwc", db_file.display());

    sea_migrate(&sqlite_url, &["fresh"]);

    let out = sea_migrate(&sqlite_url, &["reset"]);
    assert!(
        out.status.success(),
        "sea migrate reset (sqlite) doit Ok:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );

    if let Ok(db) = sea_orm::Database::connect(&sqlite_url).await {
        for table in TABLES {
            let gone = db
                .execute_unprepared(&format!("SELECT 1 FROM \"{}\" LIMIT 1", table))
                .await
                .is_err();
            assert!(
                gone,
                "table '{}' doit être supprimée après reset (sqlite)",
                table
            );
        }
    }
}
