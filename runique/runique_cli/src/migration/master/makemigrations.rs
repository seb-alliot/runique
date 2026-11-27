use super::*;

impl BuilderMaster<'_> {
    /// `makemigrations`: launches a run, once everything it needs holds. An
    /// engine in `.env` the project is not compiled for, an unreachable
    /// database, or a migration applied that `lib.rs` does not name: each
    /// stops the command before anything is read or written.
    pub(crate) async fn makemigrations(self, entities_path: &str) -> Result<()> {
        let Project { config, metadata } = read_project(self.env, self.cargo)?;
        let engine = config.engine.clone();
        let db = config.connect().await.map_err(|e| {
            anyhow::anyhow!(tf(
                runique_i18n::keys::makemigrations::error::DATABASE_UNREACHABLE,
                &[e]
            ))
        })?;
        // Before `lib.rs` is read: a run the power cut short may have left
        // it half written.
        init_migrate::repair_after_cut_short_with_entities(
            std::path::Path::new(self.migrations_path),
            std::path::Path::new(entities_path),
        )?;
        let applied = applied::read(&db).await?;
        applied::check_known(&applied, self.migrations_path)?;
        // `Pk` in a model is the project's primary key type, not this build's.
        project_pk::with_project_pk(
            entities_path,
            || project_pk::pk_of(&metadata),
            || run(entities_path, self.migrations_path, self.force, engine),
        )
    }
}
