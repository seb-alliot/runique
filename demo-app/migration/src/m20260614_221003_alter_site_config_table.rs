use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("site_config"))
                    .add_column(
                        ColumnDef::new(Alias::new("sort_order"))
                            .integer()
                            .null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("site_config"))
                    .add_column(
                        ColumnDef::new(Alias::new("is_public"))
                            .boolean()
                            .null()
                            .default(true),
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
                    .table(Alias::new("site_config"))
                    .drop_column(Alias::new("sort_order"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("site_config"))
                    .drop_column(Alias::new("is_public"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
