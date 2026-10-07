use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_test_relation_parent_tag_tag_id")
                    .table(Alias::new("test_relation_parent_tag"))
                    .col(Alias::new("tag_id"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_test_relation_parent_tag_tag_id")
                    .table(Alias::new("test_relation_parent_tag"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
