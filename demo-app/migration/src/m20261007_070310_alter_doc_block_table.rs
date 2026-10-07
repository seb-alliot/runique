use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_doc_block_page_id")
                    .table(Alias::new("doc_block"))
                    .col(Alias::new("page_id"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_doc_block_page_id")
                    .table(Alias::new("doc_block"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
