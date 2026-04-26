//! Utilitaires SQLite en mémoire pour les tests.
//!
//! Chaque appel à `fresh_db()` retourne une connexion **isolée** : les tests
//! ne se partagent jamais de données.
//!
//! # Exemple
//! ```rust
//! use crate::helpers::db;
//!
//! #[tokio::test]
//! async fn mon_test() {
//!     let db = db::fresh_db().await;
//!     db::exec(&db, "CREATE TABLE t (id INTEGER PRIMARY KEY, val TEXT)").await;
//!     db::exec(&db, "INSERT INTO t (val) VALUES ('hello')").await;
//!     db::assert_count(&db, "t", 1).await;
//! }
//! ```

use runique::sea_orm::{
    ConnectionTrait, Database, DatabaseConnection,
    sea_query::{Alias, Asterisk, Expr, Func, Query},
};

// ── Connexion ─────────────────────────────────────────────────────────────────

/// Retourne une connexion SQLite `:memory:` vide, isolée par test.
pub async fn fresh_db() -> DatabaseConnection {
    Database::connect("sqlite::memory:")
        .await
        .expect("sqlite::memory: connect")
}

/// Retourne une connexion SQLite `:memory:` avec le schéma SQL fourni déjà appliqué.
///
/// # Exemple
/// ```rust
/// let db = db::fresh_db_with_schema(
///     "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)"
/// ).await;
/// ```
pub async fn fresh_db_with_schema(schema_sql: &str) -> DatabaseConnection {
    let db = fresh_db().await;
    exec(&db, schema_sql).await;
    db
}

// ── Exécution ─────────────────────────────────────────────────────────────────

/// Exécute une instruction SQL brute et retourne le nombre de lignes affectées.
pub async fn exec(db: &DatabaseConnection, sql: &str) -> u64 {
    db.execute_unprepared(sql)
        .await
        .unwrap_or_else(|e| panic!("exec SQL échoué : {e}\n  SQL : {sql}"))
        .rows_affected()
}

/// Exécute une instruction SQL et vérifie que `n` lignes ont été affectées.
pub async fn exec_expect(db: &DatabaseConnection, sql: &str, n: u64) {
    let affected = exec(db, sql).await;
    assert_eq!(
        affected, n,
        "exec_expect : attendu {n} ligne(s) affectée(s), reçu {affected}\n  SQL : {sql}"
    );
}

// ── Comptage ──────────────────────────────────────────────────────────────────

/// Retourne le nombre de lignes dans `table`.
pub async fn count(db: &DatabaseConnection, table: &str) -> i64 {
    let stmt = Query::select()
        .expr_as(Func::count(Expr::col(Asterisk)), Alias::new("n"))
        .from(Alias::new(table))
        .to_owned();

    let row = db
        .query_one(&stmt)
        .await
        .unwrap_or_else(|e| panic!("count({table}) échoué : {e}"))
        .unwrap_or_else(|| panic!("count({table}) : aucune ligne retournée"));

    row.try_get::<i64>("", "n")
        .unwrap_or_else(|e| panic!("count({table}) : lecture échouée : {e}"))
}

/// Vérifie que la table `table` contient exactement `expected` lignes.
pub async fn assert_count(db: &DatabaseConnection, table: &str, expected: i64) {
    let n = count(db, table).await;
    assert_eq!(
        n, expected,
        "assert_count({table}) : attendu {expected} ligne(s), reçu {n}"
    );
}
