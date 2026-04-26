//! Tests SQLite en mémoire
//! Vérifie que la connexion s'ouvre, que les tables se créent et que le CRUD fonctionne.

use crate::helpers::db;

const SCHEMA: &str = "CREATE TABLE utilisateur (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    nom  TEXT    NOT NULL,
    age  INTEGER NOT NULL
)";

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_connexion_sqlite_ouvre() {
    db::fresh_db().await;
}

#[tokio::test]
async fn test_creation_de_table() {
    db::fresh_db_with_schema(SCHEMA).await;
}

#[tokio::test]
async fn test_insertion_simple() {
    let db = db::fresh_db_with_schema(SCHEMA).await;
    db::exec_expect(
        &db,
        "INSERT INTO utilisateur (nom, age) VALUES ('Alice', 30)",
        1,
    )
    .await;
}

#[tokio::test]
async fn test_insertion_multiple() {
    let db = db::fresh_db_with_schema(SCHEMA).await;
    db::exec_expect(
        &db,
        "INSERT INTO utilisateur (nom, age) VALUES ('Bob', 25), ('Charlie', 35)",
        2,
    )
    .await;
}

#[tokio::test]
async fn test_mise_a_jour() {
    let db = db::fresh_db_with_schema(SCHEMA).await;
    db::exec(
        &db,
        "INSERT INTO utilisateur (nom, age) VALUES ('Dave', 40)",
    )
    .await;
    db::exec_expect(&db, "UPDATE utilisateur SET age = 41 WHERE nom = 'Dave'", 1).await;
}

#[tokio::test]
async fn test_suppression() {
    let db = db::fresh_db_with_schema(SCHEMA).await;
    db::exec(&db, "INSERT INTO utilisateur (nom, age) VALUES ('Eve', 22)").await;
    db::exec_expect(&db, "DELETE FROM utilisateur WHERE nom = 'Eve'", 1).await;
}

#[tokio::test]
async fn test_isolation_entre_tests() {
    // Chaque test a sa propre DB :memory: — la table doit être vide au départ
    let db = db::fresh_db_with_schema(SCHEMA).await;
    db::assert_count(&db, "utilisateur", 0).await;
}
