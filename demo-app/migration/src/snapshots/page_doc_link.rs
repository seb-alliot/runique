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

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(Alias::new("page_doc_link"), Alias::new("page_id"))
                    .to(Alias::new("demo_page"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Alias::new("page_doc_link"))
                    .name("page_doc_link_page_id_demo_page_fkey")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("page_doc_link"))
                .to_owned())
            .await?;
        Ok(())
    }
}
