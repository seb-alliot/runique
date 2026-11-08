//! Helper MariaDB pour les tests d'intégration Docker.
//!
//! Nécessite que le container Docker soit actif et que `DATABASE_URL_MARIADB`
//! soit défini dans `.env.test` ou dans l'environnement.
//!
//! Si la variable est absente, les tests appelant ce module retournent
//! immédiatement (`skip` implicite — personne n'a demandé ces tests).
//! Si elle est définie mais le container n'est pas joignable, `connect()`
//! **panique** — un test qui ne peut pas vérifier ce qu'il est censé vérifier
//! ne doit pas se rapporter comme "passed". Visible par défaut, sans flag.
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
    ConnectionTrait, Database, DatabaseConnection, TransactionTrait,
    sea_query::{Alias, Asterisk, Expr, ExprTrait, Func, Query},
};

/// Charge `.env.test` si présent, puis retourne une connexion MariaDB.
/// Retourne `None` si `DATABASE_URL_MARIADB` n'est pas défini (test ignoré).
pub async fn connect() -> Option<DatabaseConnection> {
    let url = super::db_isolation::isolated_url("DATABASE_URL_MARIADB").await?;
    match Database::connect(&url).await {
        Ok(db) => Some(db),
        Err(e) => panic!(
            "DATABASE_URL_MARIADB is set ({url}) but the MariaDB container is unreachable: {e}\n\
             Run `docker compose up -d`, or unset DATABASE_URL_MARIADB in .env.test to skip MariaDB tests."
        ),
    }
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

/// Drops `eihwaz_users` and every table holding a foreign key onto it.
///
/// The test databases are shared across runs and feature sets: other tests
/// leave tables pointing at `eihwaz_users` (`eihwaz_reset_tokens`,
/// `eihwaz_sessions`, `contributions`…), their key typed by that run's
/// primary key. MariaDB keeps such a key when the parent is dropped with
/// checks off, then refuses to recreate the parent under another key type
/// (`int` → `bigint` → `binary(16)`): errno 150. Postgres has `CASCADE`.
///
/// `FOREIGN_KEY_CHECKS` is per connection and the pool hands out several:
/// the transaction keeps every statement on one.
pub async fn drop_users_and_dependents(db: &DatabaseConnection) {
    let txn = db.begin().await.expect("begin");
    txn.execute_unprepared("SET FOREIGN_KEY_CHECKS = 0")
        .await
        .expect("fk off");
    let referencing = Query::select()
        .distinct()
        .column(Alias::new("TABLE_NAME"))
        .from((
            Alias::new("information_schema"),
            Alias::new("KEY_COLUMN_USAGE"),
        ))
        .and_where(Expr::col(Alias::new("TABLE_SCHEMA")).eq(Expr::cust("DATABASE()")))
        .and_where(Expr::col(Alias::new("REFERENCED_TABLE_NAME")).eq("eihwaz_users"))
        .to_owned();
    let rows = txn
        .query_all(&referencing)
        .await
        .expect("tables referencing eihwaz_users");
    let mut tables: Vec<String> = rows
        .iter()
        .map(|row| row.try_get::<String>("", "TABLE_NAME").expect("table name"))
        .collect();
    tables.push("eihwaz_users".to_string());
    for table in tables {
        // Names read from the schema, quoted, and held to plain identifiers.
        assert!(
            table.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "unexpected table name: {table}"
        );
        txn.execute_unprepared(&format!("DROP TABLE IF EXISTS `{table}`"))
            .await
            .expect("drop");
    }
    txn.execute_unprepared("SET FOREIGN_KEY_CHECKS = 1")
        .await
        .expect("fk on");
    txn.commit().await.expect("commit");
}
