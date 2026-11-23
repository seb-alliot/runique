//! What the `runique` command does: a new project, the migrations
//! (`makemigrations`, `migration up | down | status`), the superuser, the
//! project's tests. The binary (`main.rs`) only reads the arguments.
// The parts of the framework the commands are written against, under the
// paths they had when they lived in `runique`.
pub(crate) use runique::{auth, db, utils};

pub(crate) mod atomic;
pub mod create_superuser;
pub mod migration;
pub mod new_project;
pub mod test_runner;
