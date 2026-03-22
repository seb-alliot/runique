use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("doc_block"))
                    .modify_column(ColumnDef::new(Alias::new("page_id")).integer().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("doc_block"))
                    .modify_column(ColumnDef::new(Alias::new("page_id")).integer().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
