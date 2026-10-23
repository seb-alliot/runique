use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            let exists = manager
                .get_connection()
                .query_one(
                    &Query::select()
                        .expr(Expr::val(1))
                        .from((Alias::new("pg_catalog"), Alias::new("pg_type")))
                        .and_where(Expr::col(Alias::new("typname")).eq("blocktype"))
                        .to_owned(),
                )
                .await?
                .is_some();
            if !exists {
                manager
                    .create_type(
                        sea_query::extension::postgres::Type::create()
                            .as_enum(Alias::new("blocktype"))
                            .values([
                                Alias::new("Text"),
                                Alias::new("Code"),
                                Alias::new("Sommaire"),
                            ])
                            .to_owned(),
                    )
                    .await?;
            }
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

        {
            let backs_fk = manager.get_connection().get_database_backend()
                == sea_orm::DbBackend::MySql
                && manager
                    .get_connection()
                    .query_one(
                        &Query::select()
                            .expr(Expr::val(1))
                            .from((
                                Alias::new("information_schema"),
                                Alias::new("KEY_COLUMN_USAGE"),
                            ))
                            .and_where(
                                Expr::col(Alias::new("TABLE_SCHEMA")).eq(Expr::cust("DATABASE()")),
                            )
                            .and_where(Expr::col(Alias::new("TABLE_NAME")).eq("doc_block"))
                            .and_where(Expr::col(Alias::new("COLUMN_NAME")).eq("page_id"))
                            .and_where(Expr::col(Alias::new("REFERENCED_TABLE_NAME")).is_not_null())
                            .to_owned(),
                    )
                    .await?
                    .is_some();
            if !backs_fk {
                manager
                    .create_index(
                        Index::create()
                            .name("idx_doc_block_page_id")
                            .table(Alias::new("doc_block"))
                            .col(Alias::new("page_id"))
                            .to_owned(),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("doc_block")).to_owned())
            .await?;
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .drop_type(
                    sea_query::extension::postgres::Type::drop()
                        .if_exists()
                        .name(Alias::new("blocktype"))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
