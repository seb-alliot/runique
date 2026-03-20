use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("demo_category"))
                    .add_column(ColumnDef::new(Alias::new("back_link_url")).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("demo_category"))
                    .add_column(ColumnDef::new(Alias::new("back_link_label")).string().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("demo_category"))
                    .drop_column(Alias::new("back_link_url"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("demo_category"))
                    .drop_column(Alias::new("back_link_label"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
