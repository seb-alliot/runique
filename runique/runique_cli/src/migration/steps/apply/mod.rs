//! Applying: what `migration up | down | status` does, and what follows a
//! run that succeeded (snapshots written, tags cleared, `renamed_from` taken
//! out of the models).
pub(crate) mod applied;
pub(crate) mod applied_renames;
pub mod migrate;

// For the integration tests that replay the reading on PostgreSQL and MySQL.
pub use applied::read as applied_migrations;
