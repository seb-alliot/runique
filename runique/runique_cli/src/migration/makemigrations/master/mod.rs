//! The master of `makemigrations`: what has to hold before a run, then the
//! run itself, handed to the orchestrator. It reads the project (`.env`,
//! the features in `Cargo.toml`), refuses two engines that differ, connects,
//! and refuses a database that applied a migration `lib.rs` does not name.
mod run;

use super::*;

/// What `.env` says, one key at a time.
type Env<'a> = Box<dyn Fn(&str) -> Option<String> + 'a>;

pub(crate) struct BuilderMaster<'a> {
    entities_path: &'a str,
    migrations_path: &'a str,
    force: bool,
    env: Env<'a>,
    /// Cargo's description of the project (`cargo metadata`).
    cargo: Box<dyn FnOnce() -> Result<String> + 'a>,
}

impl<'a> BuilderMaster<'a> {
    /// The project as it is: its environment, and cargo asked for real.
    pub(crate) fn new(entities_path: &'a str, migrations_path: &'a str) -> Self {
        Self {
            entities_path,
            migrations_path,
            force: false,
            env: Box::new(|key: &str| std::env::var(key).ok()),
            cargo: Box::new(project_pk::cargo_metadata),
        }
    }

    /// Lets the destructive changes through.
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
