//! Migration module — DSL definitions, schemas, columns, FKs, indexes, hooks, and utilities.
//!
//! The main entry point is [`schema::ModelSchema`], the single source of truth for a model.
//! Sub-modules expose the basic building blocks (columns, primary keys, FKs…) and the
//! diff/SQL generation utilities used by the migration engine.
pub mod changes;
pub mod cli_migration;

pub mod schema;
pub mod utils;

// Re-exports
pub use changes::*;
pub use schema::*;
pub use sea_query::ForeignKeyAction;
pub use utils::*;
