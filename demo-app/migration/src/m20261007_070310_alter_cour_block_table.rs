use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_cour_block_chapitre_id")
                    .table(Alias::new("cour_block"))
                    .col(Alias::new("chapitre_id"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_cour_block_chapitre_id")
                    .table(Alias::new("cour_block"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
