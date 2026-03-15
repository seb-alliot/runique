//! Tests — migration/migrate.rs (up, status, down + rollback Docker)
//!
//! Couverture :
//!   - up() → retourne Ok (CLI stub)
//!   - status() → liste les fichiers dans applied/
//!   - down() sans fichiers ni batch → list_available (pas de DB)
//!   - down() avec fichiers → execute SQL via Docker DB
//!   - check_order_batch / check_order_file via down() indirectement
//!   - extract_fn_block, extract_statements_from_block, seaorm_sql_type
//!     couverts indirectement par les appels down() avec Docker

use crate::helpers::db_mariadb as db_maria;
use crate::helpers::db_postgres as db_pg;
use runique::migration::migrate::{down, status, up};
use serial_test::serial;
use std::fs;
use std::path::{Path, PathBuf};

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn temp_dir(suffix: &str) -> crate::utils::clean_tpm_test::TestTempDir {
    crate::utils::clean_tpm_test::TestTempDir::new("runique_test_migrate", suffix)
}

fn applied_dir(base: &Path) -> PathBuf {
    let d = base.join("applied");
    fs::create_dir_all(&d).ok();
    d
}

/// Fichier de rollback avec DROP TABLE (pattern attendu par extract_statements_from_block)
fn down_drop_table_source(table: &str) -> String {
    format!(
        r#"
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("{table}"))
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().primary_key())
                    .to_owned(),
            )
            .await
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .drop_table(Table::drop().table(Alias::new("{table}")).to_owned())
            .await
    }}
}}
"#,
        table = table
    )
}

/// Fichier de rollback avec DROP COLUMN
fn down_drop_column_source(table: &str, col: &str) -> String {
    format!(
        r#"
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("{table}"))
                    .add_column(ColumnDef::new(Alias::new("{col}")).string().null())
                    .to_owned(),
            )
            .await
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("{table}"))
                    .drop_column(Alias::new("{col}"))
                    .to_owned(),
            )
            .await
    }}
}}
"#,
        table = table,
        col = col
    )
}

// ═══════════════════════════════════════════════════════════════
// up() — teste les deux DB (Postgres + MariaDB)
// ═══════════════════════════════════════════════════════════════

// Ignoré sur Windows avec code page non-UTF-8 (bug SQLx sur Windows français)
// Validé sur le fixe (Ryzen 7 5800X). Relancer manuellement avec : cargo test test_up -- --ignored
#[tokio::test]
#[ignore]
#[serial]
async fn test_up_retourne_ok() {
    dotenvy::from_filename(".env.test").ok();
    let pg_url = match std::env::var("DATABASE_URL_PG") {
        Ok(url) => url,
        Err(_) => return, // skip si pas de Docker
    };
    unsafe { std::env::set_var("DATABASE_URL", &pg_url) };
    let migration_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../demo-app/migration");
    let result = up(migration_dir).await;
    assert!(result.is_ok(), "up() Postgres doit Ok: {:?}", result);
}

// Ignoré sur Windows avec code page non-UTF-8 (bug SQLx sur Windows français)
// Validé sur le fixe (Ryzen 7 5800X). Relancer manuellement avec : cargo test test_up -- --ignored
#[tokio::test]
#[ignore]
#[serial]
async fn test_up_mariadb_retourne_ok() {
    dotenvy::from_filename(".env.test").ok();
    let maria_url = match std::env::var("DATABASE_URL_MARIADB") {
        Ok(url) => url,
        Err(_) => return, // skip si pas de Docker
    };
    unsafe { std::env::set_var("DATABASE_URL", &maria_url) };
    let migration_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../demo-app/migration");
    let result = up(migration_dir).await;
    assert!(result.is_ok(), "up() MariaDB doit Ok: {:?}", result);
}

#[tokio::test]
async fn test_up_chemin_inexistant_retourne_err() {
    // up() avec chemin inexistant → sea-orm-cli échoue → Err attendu
    let result = up("/chemin/inexistant/abc").await;
    assert!(result.is_err(), "up() chemin inexistant doit Err");
}

// ═══════════════════════════════════════════════════════════════
// status() — lecture fichiers, pas de DB
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_status_dossier_vide() {
    let dir = temp_dir("status_empty");
    let result = status(dir.to_str().unwrap()).await;
    assert!(
        result.is_ok(),
        "status() dossier vide doit Ok: {:?}",
        result
    );
}

#[tokio::test]
async fn test_status_dossier_inexistant() {
    let result = status("/chemin/inexistant_xyz").await;
    assert!(
        result.is_ok(),
        "status() sans applied/ doit Ok (affiche message)"
    );
}

#[tokio::test]
async fn test_status_avec_applied_vide() {
    let dir = temp_dir("status_applied_empty");
    applied_dir(&dir); // crée applied/
    let result = status(dir.to_str().unwrap()).await;
    assert!(
        result.is_ok(),
        "status() avec applied/ vide doit Ok: {:?}",
        result
    );
}

#[tokio::test]
async fn test_status_avec_table_et_fichiers() {
    let dir = temp_dir("status_files");
    let applied = applied_dir(&dir);
    let users_dir = applied.join("users");
    fs::create_dir_all(&users_dir).unwrap();
    fs::write(
        users_dir.join("20260101_120000.rs"),
        down_drop_column_source("users", "bio"),
    )
    .unwrap();

    let result = status(dir.to_str().unwrap()).await;
    assert!(
        result.is_ok(),
        "status() avec fichiers doit Ok: {:?}",
        result
    );
}

#[tokio::test]
async fn test_status_avec_by_time() {
    let dir = temp_dir("status_by_time");
    let applied = applied_dir(&dir);
    let by_time = applied.join("by_time");
    fs::create_dir_all(&by_time).unwrap();
    fs::write(
        by_time.join("20260101_120000.rs"),
        down_drop_table_source("batch_table"),
    )
    .unwrap();

    let result = status(dir.to_str().unwrap()).await;
    assert!(
        result.is_ok(),
        "status() avec by_time doit Ok: {:?}",
        result
    );
}

// ═══════════════════════════════════════════════════════════════
// down() sans fichiers → list_available, pas de DB
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_down_sans_fichiers_ni_batch_liste_available() {
    let dir = temp_dir("down_list");
    let result = down(dir.to_str().unwrap(), vec![], None).await;
    assert!(
        result.is_ok(),
        "down() sans args doit Ok (liste available): {:?}",
        result
    );
}

#[tokio::test]
async fn test_down_sans_fichiers_avec_applied() {
    let dir = temp_dir("down_list_applied");
    let applied = applied_dir(&dir);
    let users_dir = applied.join("users");
    fs::create_dir_all(&users_dir).unwrap();
    fs::write(
        users_dir.join("20260101_120000.rs"),
        down_drop_column_source("users", "bio"),
    )
    .unwrap();

    let result = down(dir.to_str().unwrap(), vec![], None).await;
    assert!(result.is_ok(), "down() liste applied doit Ok: {:?}", result);
}

// ═══════════════════════════════════════════════════════════════
// down() avec DATABASE_URL manquant → Err attendu
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_down_sans_database_url_retourne_err() {
    unsafe {
        std::env::remove_var("DATABASE_URL");
    }
    let dir = temp_dir("down_no_url");
    let applied = applied_dir(&dir);
    let users_dir = applied.join("users");
    fs::create_dir_all(&users_dir).unwrap();
    let file = users_dir.join("20260101_120000.rs");
    fs::write(&file, down_drop_column_source("users", "bio")).unwrap();

    let result = down(
        dir.to_str().unwrap(),
        vec!["users/20260101_120000".to_string()],
        None,
    )
    .await;
    assert!(result.is_err(), "down() sans DATABASE_URL doit Err");
    std::fs::remove_dir_all(&dir).ok();
}

// ═══════════════════════════════════════════════════════════════
// down() avec fichier inexistant → Err
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_down_fichier_inexistant_retourne_err() {
    // On a besoin de DATABASE_URL pour dépasser la vérification d'env,
    // mais pas d'une vraie connexion (Err attendu avant la connexion?)
    // En fait, down() vérifie le fichier APRÈS la connexion.
    // On skip si Docker pas disponible.
    let _ = dotenvy::from_filename(".env.test");
    let pg_url = match std::env::var("DATABASE_URL_PG") {
        Ok(url) => url,
        Err(_) => return,
    };
    unsafe {
        std::env::set_var("DATABASE_URL", &pg_url);
    }

    let dir = temp_dir("down_no_file");
    applied_dir(&dir);

    let result = down(
        dir.to_str().unwrap(),
        vec!["users/20260101_999999".to_string()],
        None,
    )
    .await;
    assert!(result.is_err(), "down() fichier inexistant doit Err");

    unsafe {
        std::env::remove_var("DATABASE_URL");
    }
}

// ═══════════════════════════════════════════════════════════════
// down() batch inexistant → Err
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_down_batch_inexistant_retourne_err() {
    let _ = dotenvy::from_filename(".env.test");
    let pg_url = match std::env::var("DATABASE_URL_PG") {
        Ok(url) => url,
        Err(_) => return,
    };
    unsafe {
        std::env::set_var("DATABASE_URL", &pg_url);
    }

    let dir = temp_dir("down_batch_no");
    applied_dir(&dir);

    let result = down(
        dir.to_str().unwrap(),
        vec![],
        Some("20260101_999999".to_string()),
    )
    .await;
    assert!(result.is_err(), "down() batch inexistant doit Err");

    unsafe {
        std::env::remove_var("DATABASE_URL");
    }
}

// ═══════════════════════════════════════════════════════════════
// down() avec Docker Postgres — DROP TABLE réel
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_down_drop_table_postgres() {
    let Some(db) = db_pg::connect().await else {
        return;
    };

    let pg_url = std::env::var("DATABASE_URL_PG").unwrap();
    unsafe {
        std::env::set_var("DATABASE_URL", &pg_url);
    }

    let table = "rq_test_drop_pg";
    db_pg::exec(&db, &format!("DROP TABLE IF EXISTS \"{}\"", table)).await;
    db_pg::exec(
        &db,
        &format!("CREATE TABLE \"{}\" (id SERIAL PRIMARY KEY)", table),
    )
    .await;

    let dir = temp_dir("down_drop_pg");
    let applied = applied_dir(&dir);
    let tbl_dir = applied.join(table);
    fs::create_dir_all(&tbl_dir).unwrap();
    let ts = "20260101_120000";
    fs::write(
        tbl_dir.join(format!("{}.rs", ts)),
        down_drop_table_source(table),
    )
    .unwrap();

    let result = down(
        dir.to_str().unwrap(),
        vec![format!("{}/{}", table, ts)],
        None,
    )
    .await;
    assert!(
        result.is_ok(),
        "down() DROP TABLE Postgres doit Ok: {:?}",
        result
    );

    db_pg::exec(&db, &format!("DROP TABLE IF EXISTS \"{}\"", table)).await;
    unsafe {
        std::env::remove_var("DATABASE_URL");
    }
}

#[tokio::test]
#[serial]
async fn test_down_drop_column_postgres() {
    let Some(db) = db_pg::connect().await else {
        return;
    };

    let pg_url = std::env::var("DATABASE_URL_PG").unwrap();
    unsafe {
        std::env::set_var("DATABASE_URL", &pg_url);
    }

    let table = "rq_test_drop_col_pg";
    db_pg::exec(&db, &format!("DROP TABLE IF EXISTS \"{}\"", table)).await;
    db_pg::exec(
        &db,
        &format!(
            "CREATE TABLE \"{}\" (id SERIAL PRIMARY KEY, bio TEXT)",
            table
        ),
    )
    .await;

    let dir = temp_dir("down_drop_col_pg");
    let applied = applied_dir(&dir);
    let tbl_dir = applied.join(table);
    fs::create_dir_all(&tbl_dir).unwrap();
    let ts = "20260101_130000";
    fs::write(
        tbl_dir.join(format!("{}.rs", ts)),
        down_drop_column_source(table, "bio"),
    )
    .unwrap();

    let result = down(
        dir.to_str().unwrap(),
        vec![format!("{}/{}", table, ts)],
        None,
    )
    .await;
    assert!(
        result.is_ok(),
        "down() DROP COLUMN Postgres doit Ok: {:?}",
        result
    );

    db_pg::exec(&db, &format!("DROP TABLE IF EXISTS \"{}\"", table)).await;
    unsafe {
        std::env::remove_var("DATABASE_URL");
    }
}

// ═══════════════════════════════════════════════════════════════
// down() avec Docker MariaDB — DROP TABLE réel
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_down_drop_table_mariadb() {
    let Some(db) = db_maria::connect().await else {
        return;
    };

    let mariadb_url = std::env::var("DATABASE_URL_MARIADB").unwrap();
    let mysql_url = if mariadb_url.starts_with("mysql://") {
        mariadb_url
    } else {
        mariadb_url.replacen("mariadb://", "mysql://", 1)
    };
    unsafe {
        std::env::set_var("DATABASE_URL", &mysql_url);
    }

    let table = "rq_test_drop_maria";
    db_maria::exec(&db, &format!("DROP TABLE IF EXISTS `{}`", table)).await;
    db_maria::exec(
        &db,
        &format!(
            "CREATE TABLE `{}` (id INT NOT NULL AUTO_INCREMENT PRIMARY KEY)",
            table
        ),
    )
    .await;

    let dir = temp_dir("down_drop_maria");
    let applied = applied_dir(&dir);
    let tbl_dir = applied.join(table);
    fs::create_dir_all(&tbl_dir).unwrap();
    let ts = "20260101_140000";
    fs::write(
        tbl_dir.join(format!("{}.rs", ts)),
        down_drop_table_source(table),
    )
    .unwrap();

    let result = down(
        dir.to_str().unwrap(),
        vec![format!("{}/{}", table, ts)],
        None,
    )
    .await;
    assert!(
        result.is_ok(),
        "down() DROP TABLE MariaDB doit Ok: {:?}",
        result
    );

    db_maria::exec(&db, &format!("DROP TABLE IF EXISTS `{}`", table)).await;
    unsafe {
        std::env::remove_var("DATABASE_URL");
    }
}

// ═══════════════════════════════════════════════════════════════
// down() batch — Docker Postgres
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
#[serial]
async fn test_down_batch_postgres() {
    let Some(db) = db_pg::connect().await else {
        return;
    };

    let pg_url = std::env::var("DATABASE_URL_PG").unwrap();
    unsafe {
        std::env::set_var("DATABASE_URL", &pg_url);
    }

    let table = "rq_test_batch_pg";
    db_pg::exec(&db, &format!("DROP TABLE IF EXISTS \"{}\"", table)).await;
    db_pg::exec(
        &db,
        &format!("CREATE TABLE \"{}\" (id SERIAL PRIMARY KEY)", table),
    )
    .await;

    let dir = temp_dir("down_batch_pg");
    let applied = applied_dir(&dir);
    let by_time = applied.join("by_time");
    fs::create_dir_all(&by_time).unwrap();
    let ts = "20260101_150000";
    fs::write(
        by_time.join(format!("{}.rs", ts)),
        down_drop_table_source(table),
    )
    .unwrap();

    let result = down(dir.to_str().unwrap(), vec![], Some(ts.to_string())).await;
    assert!(
        result.is_ok(),
        "down() batch Postgres doit Ok: {:?}",
        result
    );

    db_pg::exec(&db, &format!("DROP TABLE IF EXISTS \"{}\"", table)).await;
    unsafe {
        std::env::remove_var("DATABASE_URL");
    }
}
