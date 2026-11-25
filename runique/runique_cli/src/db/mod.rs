//! The connection to the project's database, by the application's own rule:
//! `DATABASE_URL`, then the `DB_*` keys. Copied from `runique::db`, so that
//! the command builds without the framework.
pub mod builder;
pub mod config;
pub mod engine;
pub mod sql_dialect;

pub use builder::DatabaseConfigBuilder;
pub use config::DatabaseConfig;
pub use engine::DatabaseEngine;
