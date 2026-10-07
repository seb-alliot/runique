use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_demo_page_category_id")
                    .table(Alias::new("demo_page"))
                    .col(Alias::new("category_id"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_demo_page_category_id")
                    .table(Alias::new("demo_page"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
