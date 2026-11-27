//! The migration engine of the `runique` command. `makemigrations` goes
//! through four steps, each in its folder under `steps`:
//!
//! - `parser`: reads the models (the DSL, through `runique_dsl`) and reads the
//!   snapshots back; both give the description of a table;
//! - `diff`: compares two descriptions and says what changed;
//! - `changes`: checks what changed (destructive elements, names, primary
//!   key) and puts the tables in order;
//! - `generate`: writes the migration files, one function per case.
//!
//! `steps::apply` is what `migration up` does. Around them: the
//! `makemigrations` command, which goes through the steps, and the functions
//! the generated migrations call when they run (`runtime`). The schema of a model, which an
//! application runs with, is `runique::schema`.
pub mod steps;

pub mod helpers;
pub mod makemigrations;
pub(crate) mod master;
pub mod paths;
pub mod runtime;
pub mod types;

#[cfg(test)]
mod tests;

// Re-exports
pub use helpers::{db_columns, has_marker, without_marker};
pub use makemigrations::{makemigrations, run};
pub use paths::*;
pub use sea_query::ForeignKeyAction;
pub use steps::apply::applied_migrations;
pub use steps::changes::*;
pub use steps::diff::*;
pub use steps::generate::*;
pub use steps::parser::*;
pub use types::*;
