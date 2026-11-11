//! Migration module — DSL definitions, schemas, columns, FKs, indexes, hooks, and utilities.
//!
//! The main entry point is [`schema::ModelSchema`], the single source of truth for a model.
//! Sub-modules expose the basic building blocks (columns, primary keys, FKs…) and the
//! diff/SQL generation utilities used by the migration engine.
pub mod changes;
pub mod cli_migration;
pub mod generate;
pub mod parse;

pub mod diff;
pub mod helpers;
pub mod paths;
pub mod schema;
pub mod types;

#[cfg(test)]
mod tests_pipeline;

// Re-exports
pub use changes::*;
pub use diff::*;
pub use generate::*;
pub use helpers::*;
pub use parse::*;
pub use paths::*;
pub use schema::*;
pub use sea_query::ForeignKeyAction;
pub use types::*;
