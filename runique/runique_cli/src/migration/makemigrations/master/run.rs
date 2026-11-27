use super::*;

impl BuilderMaster<'_> {
    /// Launches a run, once everything it needs holds. An engine in `.env`
    /// the project is not compiled for, an unreachable database, or a
    /// migration applied that `lib.rs` does not name: each stops the command
    /// before anything is read or written.
    pub(crate) async fn run(self) -> Result<()> {
        let mut config = DatabaseConfig::from_lookup(self.env)
            .map_err(anyhow::Error::msg)?
            .build();
        // The command only ever needs one connection.
        config.pool(1, 1);
        let engine = config.engine.clone();
        // Cargo is asked once: the engine the project is compiled for now,
        // the type of `Pk` below.
        let metadata = (self.cargo)()?;
        project_engine::check(&engine, &project_pk::features_of(&metadata)?)?;
        let db = config.connect().await.map_err(|e| {
            anyhow::anyhow!(tf(
                runique_i18n::keys::makemigrations::error::DATABASE_UNREACHABLE,
                &[e]
            ))
        })?;
        // Before `lib.rs` is read: a run the power cut short may have left
        // it half written.
        init_migrate::repair_beside(
            std::path::Path::new(self.migrations_path),
            std::path::Path::new(self.entities_path),
        )?;
        let applied = applied::read(&db).await?;
        applied::check_known(&applied, self.migrations_path)?;
        // `Pk` in a model is the project's primary key type, not this build's.
        project_pk::with_project_pk(
            self.entities_path,
            || project_pk::pk_of(&metadata),
            || run(self.entities_path, self.migrations_path, self.force, engine),
        )
    }
}
