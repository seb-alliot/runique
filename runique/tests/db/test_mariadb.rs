//! Tests MariaDB via Docker
//! Nécessite `docker compose up -d` et DATABASE_URL_MARIADB dans .env.test
//! Si la variable est absente, chaque test retourne immédiatement (skip implicite).
//! Les tests tournent en série (#[serial]) car ils partagent la même base Docker.

use crate::helpers::db_mariadb as db;
use serial_test::serial;

const SCHEMA: &str = "CREATE TABLE utilisateur (
    id   INT AUTO_INCREMENT PRIMARY KEY,
    nom  VARCHAR(255) NOT NULL,
    age  INT NOT NULL
)";

const DROP: &str = "DROP TABLE IF EXISTS utilisateur";

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
#[serial]
async fn test_connexion_mariadb_ouvre() {
    let Some(_db) = db::connect().await else {
        return;
    };
}

#[tokio::test]
#[serial]
async fn test_creation_de_table_mariadb() {
    let Some(db) = db::connect().await else {
        return;
    };
    db::exec(&db, DROP).await;
    db::exec(&db, SCHEMA).await;
    db::exec(&db, DROP).await;
}

#[tokio::test]
#[serial]
async fn test_insertion_simple_mariadb() {
    let Some(db) = db::connect().await else {
        return;
    };
    db::exec(&db, DROP).await;
    db::exec(&db, SCHEMA).await;
    db::exec_expect(
        &db,
        "INSERT INTO utilisateur (nom, age) VALUES ('Alice', 30)",
        1,
    )
    .await;
    db::assert_count(&db, "utilisateur", 1).await;
    db::exec(&db, DROP).await;
}

#[tokio::test]
#[serial]
async fn test_insertion_multiple_mariadb() {
    let Some(db) = db::connect().await else {
        return;
    };
    db::exec(&db, DROP).await;
    db::exec(&db, SCHEMA).await;
    db::exec_expect(
        &db,
        "INSERT INTO utilisateur (nom, age) VALUES ('Bob', 25), ('Charlie', 35)",
        2,
    )
    .await;
    db::assert_count(&db, "utilisateur", 2).await;
    db::exec(&db, DROP).await;
}

#[tokio::test]
#[serial]
async fn test_mise_a_jour_mariadb() {
    let Some(db) = db::connect().await else {
        return;
    };
    db::exec(&db, DROP).await;
    db::exec(&db, SCHEMA).await;
    db::exec(
        &db,
        "INSERT INTO utilisateur (nom, age) VALUES ('Dave', 40)",
    )
    .await;
    db::exec_expect(&db, "UPDATE utilisateur SET age = 41 WHERE nom = 'Dave'", 1).await;
    db::exec(&db, DROP).await;
}

#[tokio::test]
#[serial]
async fn test_suppression_mariadb() {
    let Some(db) = db::connect().await else {
        return;
    };
    db::exec(&db, DROP).await;
    db::exec(&db, SCHEMA).await;
    db::exec(&db, "INSERT INTO utilisateur (nom, age) VALUES ('Eve', 22)").await;
    db::exec_expect(&db, "DELETE FROM utilisateur WHERE nom = 'Eve'", 1).await;
    db::exec(&db, DROP).await;
}
