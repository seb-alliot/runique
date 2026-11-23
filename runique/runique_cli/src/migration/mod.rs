//! The migration engine of the `runique` command: reading the models, the
//! diff against their snapshots, the generated migration files, and the
//! commands themselves (`cli_migration`).
//!
//! What an application runs with (the schema of a model, the functions the
//! generated migrations call) stays in `runique::migration`, re-exported here.
pub mod changes;
pub mod cli_migration;
pub mod generate;
pub mod parse;

pub mod diff;
pub mod helpers;
pub mod paths;
pub mod types;

pub use runique::migration::{runtime, schema};

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
