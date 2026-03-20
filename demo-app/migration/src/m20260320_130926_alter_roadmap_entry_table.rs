use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("roadmap_entry"))
                    .add_column(ColumnDef::new(Alias::new("link_url_2")).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("roadmap_entry"))
                    .add_column(ColumnDef::new(Alias::new("link_label_2")).string().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("roadmap_entry"))
                    .drop_column(Alias::new("link_url_2"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("roadmap_entry"))
                    .drop_column(Alias::new("link_label_2"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
