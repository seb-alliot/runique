//! The migration engine of the `runique` command: reading the models, the
//! diff against their snapshots, the generated migration files, and the
//! commands themselves (`cli_migration`).
//!
//! The functions the generated migrations call are in `runtime`. The schema of
//! a model, which an application runs with, is `runique::schema`.
pub mod changes;
pub mod cli_migration;
pub mod generate;
pub mod helpers;
pub mod parse;

pub mod diff;
pub mod paths;
pub mod runtime;
pub mod types;

#[cfg(test)]
mod tests_pipeline;

// Re-exports
pub use changes::*;
pub use diff::*;
pub use generate::*;
pub use helpers::{db_columns, decimal_may_shrink, has_marker, length_may_shrink, without_marker};
pub use parse::*;
pub use paths::*;
pub use sea_query::ForeignKeyAction;
pub use types::*;
