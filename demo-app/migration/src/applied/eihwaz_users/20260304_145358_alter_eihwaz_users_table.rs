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
                    .drop_column(Alias::new("updated_at"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .drop_column(Alias::new("created_at"))
                    .to_owned(),
            )
            .await?;

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
                    .modify_column(ColumnDef::new(Alias::new("username")).string().not_null().unique_key())
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
                    .modify_column(ColumnDef::new(Alias::new("email")).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .modify_column(ColumnDef::new(Alias::new("username")).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .add_column(ColumnDef::new(Alias::new("updated_at")).date_time().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .add_column(ColumnDef::new(Alias::new("created_at")).date_time().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
