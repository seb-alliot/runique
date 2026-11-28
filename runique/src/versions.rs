//! What this crate is built with, and what the `runique` command (the
//! `runique_cli` crate) reads from it.

/// The version of this crate: the one `runique new` and `makemigrations`
/// write as the `runique` dependency of a project.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The SeaORM requirements this crate is built with (read from the manifests
/// by `build.rs`): a generated project pins the same.
pub const SEA_ORM_REQ: &str = env!("RUNIQUE_SEA_ORM_REQ");
pub const SEA_ORM_MIGRATION_REQ: &str = env!("RUNIQUE_SEA_ORM_MIGRATION_REQ");

/// Set by `runique test` when it writes to a terminal. Its output goes through
/// a pipe, so the test code can't tell it's a terminal and would drop colors.
pub const TEST_COLOR_ENV: &str = "RUNIQUE_TEST_COLOR";
