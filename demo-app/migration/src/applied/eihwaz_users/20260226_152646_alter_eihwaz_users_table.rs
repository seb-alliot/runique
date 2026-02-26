use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .modify_column(ColumnDef::new(Alias::new("email")).string().not_null().unique_key())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .add_column(ColumnDef::new(Alias::new("is_superuser")).boolean().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .add_column(ColumnDef::new(Alias::new("is_staff")).boolean().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .add_column(ColumnDef::new(Alias::new("roles")).string().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .drop_column(Alias::new("is_superuser"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .drop_column(Alias::new("is_staff"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .drop_column(Alias::new("roles"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .modify_column(ColumnDef::new(Alias::new("email")).string().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
