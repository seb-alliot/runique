//! Migration module — DSL definitions, schemas, columns, FKs, indexes, hooks, relations, and utilities.
//!
//! The main entry point is [`schema::ModelSchema`], the single source of truth for a model.
//! Sub-modules expose the basic building blocks (columns, primary keys, FKs…) and the
//! diff/SQL generation utilities used by the migration engine.
pub mod column;
pub mod foreign_key;
pub mod hooks;
pub mod index;

pub mod primary_key;
pub mod relation;
pub mod schema;
pub mod utils;

// Re-exports
pub use column::*;
pub use foreign_key::*;
pub use hooks::*;
pub use index::*;
pub use primary_key::*;
pub use relation::*;
pub use schema::*;
pub use utils::*;
