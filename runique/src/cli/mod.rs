//! Runique CLI commands — new project, migration generation, superuser, tests.
pub mod new_project;
pub use new_project::create_new_project;

// The `makemigrations` command lives in `migration/cli_migration`; this name
// keeps `runique::cli::makemigration::…` working for the binary and the tests.
pub use crate::migration::cli_migration as makemigration;
pub use makemigration::{
    scan_entities, seaorm_alter_file_path, seaorm_alter_module_name, update_migration_lib,
};

pub mod migrate;
pub(crate) mod migration_crate;

pub mod cli_admin;
pub use cli_admin::create_superuser;

pub mod test_runner;
pub use test_runner::run_tests;
