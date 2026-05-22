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
                    .add_column(ColumnDef::new(Alias::new("bio")).text().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .add_column(ColumnDef::new(Alias::new("avatar")).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .add_column(ColumnDef::new(Alias::new("website")).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .add_column(ColumnDef::new(Alias::new("phone")).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .add_column(ColumnDef::new(Alias::new("birth_date")).date().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .add_column(ColumnDef::new(Alias::new("is_verified")).boolean().null())
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
                    .drop_column(Alias::new("bio"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .drop_column(Alias::new("avatar"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .drop_column(Alias::new("website"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .drop_column(Alias::new("phone"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .drop_column(Alias::new("birth_date"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("eihwaz_users"))
                    .drop_column(Alias::new("is_verified"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
