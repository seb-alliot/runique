use super::*;

impl BuilderMaster<'_> {
    /// `migration up`: applies every pending migration, then records what
    /// was applied (`migrate::record_applied`).
    pub(crate) async fn migration_up(
        self,
        entities_path: &str,
        migrations_path: &str,
    ) -> Result<()> {
        let metadata = self.migration(migrations_path, &["up"]).await?;
        // The snapshots are written from the models: `Pk` as the project
        // has it.
        project_pk::with_project_pk(
            entities_path,
            || project_pk::pk_of(&metadata),
            || migrate::record_applied(entities_path, migrations_path),
        )
    }

    /// `migration down`: rolls back the last `n` applied migrations.
    pub(crate) async fn migration_down(self, migrations_path: &str, n: u32) -> Result<()> {
        self.migration(migrations_path, &["down", "-n", &n.to_string()])
            .await
            .map(drop)
    }

    /// `migration status`: lists the migrations and whether each is applied.
    pub(crate) async fn migration_status(self, migrations_path: &str) -> Result<()> {
        self.migration(migrations_path, &["status"]).await.map(drop)
    }

    /// Launches the project's migration crate with `args`. What can't run is
    /// refused first, before cargo is asked anything: no crate, or a
    /// `makemigrations` cut short. Gives back cargo's description of the
    /// project.
    async fn migration(self, migrations_path: &str, args: &[&str]) -> Result<String> {
        migrate::runnable(migrations_path)?;
        let Project { config, metadata } = read_project(self.env, self.cargo)?;
        migrate::launch(migrations_path, args, &config).await?;
        Ok(metadata)
    }
}
