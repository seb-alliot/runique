//! What the `runique` command does: a new project, the migrations
//! (`makemigrations`, `migration up | down | status`), the superuser, the
//! project's tests. The binary (`main.rs`) only reads the arguments.
// The parts of the framework the commands are written against, under the
// paths they had when they lived in `runique`: the command's own copies, so
// that it builds without the framework.
pub(crate) mod atomic;
pub(crate) mod auth;
pub mod create_superuser;
pub mod db;
pub mod migration;
pub mod new_project;
pub mod test_runner;
pub(crate) mod tooling;
pub mod utils;
