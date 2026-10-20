//! Runique CLI commands — new project, migration generation, superuser, tests.
pub mod new_project;
pub use new_project::create_new_project;

pub mod makemigration;
pub use makemigration::{
    scan_entities, seaorm_alter_file_path, seaorm_alter_module_name, update_migration_lib,
};

pub mod migrate;

pub mod cli_admin;
pub use cli_admin::create_superuser;

pub mod test_runner;
pub use test_runner::run_tests;
