use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("page_doc_link"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("page_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("label")).string().not_null())
                    .col(ColumnDef::new(Alias::new("url")).string().not_null())
                    .col(ColumnDef::new(Alias::new("link_type")).string().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).integer().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("page_doc_link"))
                .to_owned())
            .await?;
        Ok(())
    }
}
