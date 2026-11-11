//! Runique CLI commands — new project, migration generation and application,
//! superuser, tests. The `makemigrations` logic lives in `migration::cli_migration`.
pub mod new_project;
pub use new_project::create_new_project;

pub mod makemigration;
pub mod migrate;

pub mod create_superuser;
pub use create_superuser::create_superuser;

pub mod test_runner;
pub use test_runner::run_tests;
