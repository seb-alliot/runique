use super::*;

impl BuilderMaster<'_> {
    /// `create-superuser`: reads the project and connects, then hands the
    /// database to the wizard. The key of the accounts table follows a
    /// feature of the project (`big-pk`, `pk-uuid`), not one of this build:
    /// it comes from the same answer of cargo as the engine.
    pub(crate) async fn create_superuser(self) -> Result<()> {
        let Project { config, metadata } = read_project(self.env, self.cargo)?;
        let pk = project_pk::pk_of(&metadata)?;
        let db = config.connect().await.map_err(|e| {
            anyhow::anyhow!(tf(
                runique_i18n::keys::makemigrations::error::DATABASE_UNREACHABLE,
                &[e]
            ))
        })?;
        crate::create_superuser::wizard(&db, pk).await
    }
}
