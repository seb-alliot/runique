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
                    .add_column(
                        ColumnDef::new(Alias::new("chapitre_id"))
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_doc_block_chapitre")
                    .from(Alias::new("doc_block"), Alias::new("chapitre_id"))
                    .to(Alias::new("chapitre"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_doc_block_chapitre")
                    .table(Alias::new("doc_block"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("doc_block"))
                    .drop_column(Alias::new("chapitre_id"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
