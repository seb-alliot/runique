//! Helper MariaDB pour les tests d'intégration Docker.
//!
//! Nécessite que le container Docker soit actif et que `DATABASE_URL_MARIADB`
//! soit défini dans `.env.test` ou dans l'environnement.
//!
//! Si la variable est absente, les tests appelant ce module retournent
//! immédiatement (`skip` implicite).
//!
//! # Lancer les containers
//! ```bash
//! docker compose up -d
//! ```
//!
//! # Exemple
//! ```rust
//! #[tokio::test]
//! async fn mon_test_mariadb() {
//!     let Some(db) = db_mariadb::connect().await else { return };
//!     db_mariadb::exec(&db, "CREATE TABLE t (id INT AUTO_INCREMENT PRIMARY KEY, val TEXT)").await;
//!     db_mariadb::exec(&db, "INSERT INTO t (val) VALUES ('hello')").await;
//!     db_mariadb::assert_count(&db, "t", 1).await;
//! }
//! ```

use runique::sea_orm::{
    sea_query::{Alias, Asterisk, Expr, Func, Query},
    ConnectionTrait, Database, DatabaseConnection,
};

/// Charge `.env.test` si présent, puis retourne une connexion MariaDB.
/// Retourne `None` si `DATABASE_URL_MARIADB` n'est pas défini (test ignoré).
pub async fn connect() -> Option<DatabaseConnection> {
    let _ = dotenvy::from_filename(".env.test");
    let url = std::env::var("DATABASE_URL_MARIADB").ok()?;
    Database::connect(&url).await.ok()
}

/// Exécute une instruction SQL brute et retourne le nombre de lignes affectées.
pub async fn exec(db: &DatabaseConnection, sql: &str) -> u64 {
    db.execute_unprepared(sql)
        .await
        .unwrap_or_else(|e| panic!("exec SQL (mariadb) échoué : {e}\n  SQL : {sql}"))
        .rows_affected()
}

/// Exécute une instruction SQL et vérifie que `n` lignes ont été affectées.
pub async fn exec_expect(db: &DatabaseConnection, sql: &str, n: u64) {
    let affected = exec(db, sql).await;
    assert_eq!(
        affected, n,
        "exec_expect (mariadb) : attendu {n} ligne(s), reçu {affected}\n  SQL : {sql}"
    );
}

/// Retourne le nombre de lignes dans `table`.
pub async fn count(db: &DatabaseConnection, table: &str) -> i64 {
    let stmt = Query::select()
        .expr_as(Func::count(Expr::col(Asterisk)), Alias::new("n"))
        .from(Alias::new(table))
        .to_owned();

    let row = db
        .query_one(&stmt)
        .await
        .unwrap_or_else(|e| panic!("count(mariadb:{table}) échoué : {e}"))
        .unwrap_or_else(|| panic!("count(mariadb:{table}) : aucune ligne retournée"));

    row.try_get::<i64>("", "n")
        .unwrap_or_else(|e| panic!("count(mariadb:{table}) : lecture échouée : {e}"))
}

/// Vérifie que la table `table` contient exactement `expected` lignes.
pub async fn assert_count(db: &DatabaseConnection, table: &str, expected: i64) {
    let n = count(db, table).await;
    assert_eq!(
        n, expected,
        "assert_count(mariadb:{table}) : attendu {expected} ligne(s), reçu {n}"
    );
}
