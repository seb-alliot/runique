//! Migration module — what an application runs with: the schema of a model
//! and the functions the generated migrations call.
//!
//! The main entry point is [`schema::ModelSchema`], the single source of truth for a model.
//! Reading the models, comparing them and writing the migration files is the
//! `runique` command's work: the `runique_cli` crate.
pub mod runtime;
pub mod schema;

// Re-exports
pub use schema::*;
pub use sea_query::ForeignKeyAction;
