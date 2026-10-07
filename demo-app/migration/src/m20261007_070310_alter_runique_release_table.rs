use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("runique_release"))
                    .add_column(
                        ColumnDef::new(Alias::new("changelog_en_url"))
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("runique_release"))
                    .add_column(
                        ColumnDef::new(Alias::new("changelog_fr_url"))
                            .string()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("runique_release"))
                    .drop_column(Alias::new("changelog_en_url"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("runique_release"))
                    .drop_column(Alias::new("changelog_fr_url"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
