use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("doc_block"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("page_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("heading")).string().null())
                    .col(ColumnDef::new(Alias::new("content")).string().not_null())
                    .col(ColumnDef::new(Alias::new("block_type")).string().not_null())
                    .col(
                        ColumnDef::new(Alias::new("sort_order"))
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        if manager.get_database_backend() != DbBackend::Sqlite {
            manager
                .create_foreign_key(
                    ForeignKey::create()
                        .from(Alias::new("doc_block"), Alias::new("page_id"))
                        .to(Alias::new("doc_page"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::NoAction)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_database_backend() != DbBackend::Sqlite {
            manager
                .drop_foreign_key(
                    ForeignKey::drop()
                        .table(Alias::new("doc_block"))
                        .name("doc_block_page_id_doc_page_fkey")
                        .to_owned(),
                )
                .await?;
        }

        manager
            .drop_table(Table::drop().table(Alias::new("doc_block")).to_owned())
            .await?;
        Ok(())
    }
}
