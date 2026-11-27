use super::*;

/// What a command knows of the project before it connects.
pub(super) struct Project {
    /// The database `.env` names, by the app's own rule.
    pub(super) config: DatabaseConfig,
    /// Cargo's description of the project: its features, the type of `Pk`.
    pub(super) metadata: String,
}

/// Reads the project, and refuses an engine in `.env` that it is not
/// compiled for. Nothing is connected to yet, nothing is written.
///
/// Cargo is asked once: the engine the project is compiled for here, the
/// type of `Pk` later, from the same answer.
pub(super) fn read_project(env: Env<'_>, cargo: Cargo<'_>) -> Result<Project> {
    let config = migrate::configuration(env)?;
    let metadata = cargo()?;
    project_engine::check(&config.engine, &project_pk::features_of(&metadata)?)?;
    Ok(Project { config, metadata })
}
