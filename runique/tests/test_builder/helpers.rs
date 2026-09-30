//! A throwaway SQLite database per test, an env file pointing to it, and a
//! connection that sits outside the test transaction to see what really stayed.
use runique::sea_orm::{
    ConnectionTrait, Database, DatabaseConnection, DbErr, Statement,
    sea_query::{Alias, Query},
};
use std::path::PathBuf;

pub struct Scratch {
    dir: PathBuf,
}

impl Scratch {
    /// A fresh directory, named after the test so a leftover one is easy to trace.
    pub fn new(name: &str) -> Self {
        let dir =
            std::env::temp_dir().join(format!("runique_builder_{}_{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        Self { dir }
    }

    pub fn sqlite_url(&self) -> String {
        format!(
            "sqlite://{}?mode=rwc",
            self.dir.join("test.sqlite").display()
        )
    }

    /// Writes an env file with `lines` and returns its absolute path.
    pub fn env_file(&self, lines: &[String]) -> String {
        let path = self.dir.join("test.env");
        std::fs::write(&path, lines.join("\n")).expect("env file");
        path.display().to_string()
    }

    /// The usual env file: this scratch's SQLite database, plus `extra` lines.
    pub fn sqlite_env(&self, extra: &[&str]) -> String {
        let mut lines = vec![format!("DATABASE_URL={}", self.sqlite_url())];
        lines.extend(extra.iter().map(|l| l.to_string()));
        self.env_file(&lines)
    }

    /// A connection of its own, outside any test transaction, with the
    /// `items` table created.
    pub async fn outside(&self) -> DatabaseConnection {
        let conn = Database::connect(self.sqlite_url())
            .await
            .expect("sqlite connect");
        create_items(&conn).await;
        conn
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

pub const ITEMS: &str = "rq_builder_items";

pub async fn create_items(conn: &DatabaseConnection) {
    let sql = match conn.get_database_backend() {
        runique::sea_orm::DbBackend::Postgres => {
            "CREATE TABLE IF NOT EXISTS rq_builder_items (id SERIAL PRIMARY KEY, name VARCHAR(50) NOT NULL UNIQUE)"
        }
        runique::sea_orm::DbBackend::MySql => {
            "CREATE TABLE IF NOT EXISTS rq_builder_items (id INT AUTO_INCREMENT PRIMARY KEY, name VARCHAR(50) NOT NULL UNIQUE)"
        }
        _ => {
            "CREATE TABLE IF NOT EXISTS rq_builder_items (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE)"
        }
    };
    conn.execute_unprepared(sql).await.expect("create items");
}

/// Inserts one row through a query SeaORM builds, so it shows in the trace.
pub async fn insert(db: &impl ConnectionTrait, name: &str) -> Result<(), DbErr> {
    let stmt = Query::insert()
        .into_table(Alias::new(ITEMS))
        .columns([Alias::new("name")])
        .values_panic([name.into()])
        .to_owned();
    db.execute(&stmt).await.map(|_| ())
}

pub async fn count(db: &impl ConnectionTrait) -> i64 {
    let backend = db.get_database_backend();
    db.query_one_raw(Statement::from_string(
        backend,
        format!("SELECT COUNT(*) AS n FROM {ITEMS}"),
    ))
    .await
    .expect("count query")
    .expect("count row")
    .try_get::<i64>("", "n")
    .expect("count value")
}
