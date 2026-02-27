//! Tests SQLite en mémoire
//! Vérifie que la connexion s'ouvre, que les tables se créent et que le CRUD fonctionne.

use runique::sea_orm::{ConnectionTrait, Database};

// ── Helper ────────────────────────────────────────────────────────────────────

async fn setup_db() -> runique::sea_orm::DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Connexion SQLite en mémoire échouée");

    db.execute_unprepared(
        "CREATE TABLE utilisateur (
            id   INTEGER PRIMARY KEY AUTOINCREMENT,
            nom  TEXT    NOT NULL,
            age  INTEGER NOT NULL
        )",
    )
    .await
    .expect("Création de table échouée");

    db
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_connexion_sqlite_ouvre() {
    Database::connect("sqlite::memory:")
        .await
        .expect("La connexion SQLite doit s'ouvrir");
}

#[tokio::test]
async fn test_creation_de_table() {
    // Si execute_unprepared ne panique pas, la table a bien été créée
    let _db = setup_db().await;
}

#[tokio::test]
async fn test_insertion_simple() {
    let db = setup_db().await;

    let result = db
        .execute_unprepared("INSERT INTO utilisateur (nom, age) VALUES ('Alice', 30)")
        .await
        .expect("INSERT échoué");

    assert_eq!(result.rows_affected(), 1, "L'INSERT doit affecter 1 ligne");
}

#[tokio::test]
async fn test_insertion_multiple() {
    let db = setup_db().await;

    let result = db
        .execute_unprepared(
            "INSERT INTO utilisateur (nom, age) VALUES ('Bob', 25), ('Charlie', 35)",
        )
        .await
        .expect("INSERT multiple échoué");

    assert_eq!(result.rows_affected(), 2, "L'INSERT doit affecter 2 lignes");
}

#[tokio::test]
async fn test_mise_a_jour() {
    let db = setup_db().await;

    db.execute_unprepared("INSERT INTO utilisateur (nom, age) VALUES ('Dave', 40)")
        .await
        .expect("INSERT échoué");

    let result = db
        .execute_unprepared("UPDATE utilisateur SET age = 41 WHERE nom = 'Dave'")
        .await
        .expect("UPDATE échoué");

    assert_eq!(result.rows_affected(), 1, "L'UPDATE doit affecter 1 ligne");
}

#[tokio::test]
async fn test_suppression() {
    let db = setup_db().await;

    db.execute_unprepared("INSERT INTO utilisateur (nom, age) VALUES ('Eve', 22)")
        .await
        .expect("INSERT échoué");

    let result = db
        .execute_unprepared("DELETE FROM utilisateur WHERE nom = 'Eve'")
        .await
        .expect("DELETE échoué");

    assert_eq!(result.rows_affected(), 1, "Le DELETE doit affecter 1 ligne");
}

#[tokio::test]
async fn test_isolation_entre_tests() {
    // Chaque test a sa propre DB :memory: — la table doit être vide au départ
    let db = setup_db().await;

    let result = db
        .execute_unprepared("DELETE FROM utilisateur")
        .await
        .expect("DELETE échoué");

    assert_eq!(result.rows_affected(), 0, "La DB doit être vide au départ");
}
