use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `CREATE TYPE` is Postgres-only syntax — `ColumnType::Enum` below needs no
        // such guard, sea-query already renders it correctly per backend on its own.
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager.get_connection().execute_unprepared(
                "DO $$ BEGIN CREATE TYPE BlockType AS ENUM ('Text', 'Code', 'Sommaire'); EXCEPTION WHEN duplicate_object THEN NULL; END $$"
            ).await?;
        }

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
                    .col(ColumnDef::new(Alias::new("content")).text().not_null())
                    .col(
                        ColumnDef::new_with_type(
                            Alias::new("block_type"),
                            ColumnType::Enum {
                                name: Alias::new("BlockType").into_iden(),
                                variants: vec![
                                    Alias::new("Text").into_iden(),
                                    Alias::new("Code").into_iden(),
                                    Alias::new("Sommaire").into_iden(),
                                ],
                            },
                        )
                        .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("sort_order"))
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("doc_block_page_id_doc_page_fkey")
                            .from(Alias::new("doc_block"), Alias::new("page_id"))
                            .to(Alias::new("doc_page"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("doc_block")).to_owned())
            .await?;
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared("DROP TYPE IF EXISTS BlockType")
                .await?;
        }

        Ok(())
    }
}
