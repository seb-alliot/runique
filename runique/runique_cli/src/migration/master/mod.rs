//! The master of the commands that work on a project's database: what has
//! to hold before any of them starts, read once and the same way for all.
//! It reads the project (`.env`, the features of `runique` in `Cargo.toml`)
//! and refuses two engines that differ; each command then does its own
//! work: `makemigrations` hands a run to the orchestrator, `migration up |
//! down | status` launches the project's migration crate.
mod makemigrations;
mod migration;
mod project;

use crate::db::DatabaseConfig;
use crate::migration::makemigrations::run;
use crate::migration::steps::apply::{applied, migrate};
use crate::migration::steps::generate::init_migrate;
use crate::migration::steps::parser::files::{project_engine, project_pk};
use anyhow::Result;
use runique_i18n::tf;

use project::{Project, read_project};

/// What `.env` says, one key at a time.
type Env<'a> = Box<dyn Fn(&str) -> Option<String> + 'a>;
/// Cargo's description of the project (`cargo metadata`).
type Cargo<'a> = Box<dyn FnOnce() -> Result<String> + 'a>;

pub(crate) struct BuilderMaster<'a> {
    migrations_path: &'a str,
    force: bool,
    env: Env<'a>,
    cargo: Cargo<'a>,
}

impl<'a> BuilderMaster<'a> {
    /// The project as it is: its environment, and cargo asked for real.
    pub(crate) fn new(migrations_path: &'a str) -> Self {
        Self {
            migrations_path,
            force: false,
            env: Box::new(|key: &str| std::env::var(key).ok()),
            cargo: Box::new(project_pk::cargo_metadata),
        }
    }

    /// Lets the destructive changes of `makemigrations` through.
    pub(crate) fn force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    /// Another environment than the one of the process.
    #[cfg(test)]
    pub(crate) fn env(mut self, env: impl Fn(&str) -> Option<String> + 'a) -> Self {
        self.env = Box::new(env);
        self
    }

    /// Another description of the project than the one cargo gives.
    #[cfg(test)]
    pub(crate) fn cargo(mut self, cargo: impl FnOnce() -> Result<String> + 'a) -> Self {
        self.cargo = Box::new(cargo);
        self
    }
}
