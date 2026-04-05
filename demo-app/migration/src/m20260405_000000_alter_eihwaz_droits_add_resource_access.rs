use sea_orm_migration::prelude::*;

/// Ajoute `resource_key` et `access_type` sur `eihwaz_droits`.
///
/// - `resource_key NULL` → droit global (comportement existant inchangé)
/// - `resource_key = "blog"` + `access_type = "view"` → voir la ressource dans la nav
/// - `resource_key = "blog"` + `access_type = "write"` → create/edit/delete
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_droits"))
                    .add_column(ColumnDef::new(Alias::new("resource_key")).string().null())
                    .add_column(ColumnDef::new(Alias::new("access_type")).string().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_droits"))
                    .drop_column(Alias::new("resource_key"))
                    .drop_column(Alias::new("access_type"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
