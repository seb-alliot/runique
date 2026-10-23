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
                        .and_where(Expr::col(Alias::new("typname")).eq("pagetype"))
                        .to_owned(),
                )
                .await?
                .is_some();
            if !exists {
                manager
                    .create_type(
                        sea_query::extension::postgres::Type::create()
                            .as_enum(Alias::new("pagetype"))
                            .values([
                                Alias::new("code"),
                                Alias::new("form"),
                                Alias::new("custom"),
                                Alias::new("doc_en"),
                                Alias::new("doc_fr"),
                            ])
                            .to_owned(),
                    )
                    .await?;
            }
        }

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("demo_page"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("category_id"))
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("slug")).string().not_null())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(ColumnDef::new(Alias::new("lead")).string().null())
                    .col(
                        ColumnDef::new_with_type(
                            Alias::new("page_type"),
                            ColumnType::Enum {
                                name: Alias::new("PageType").into_iden(),
                                variants: vec![
                                    Alias::new("code").into_iden(),
                                    Alias::new("form").into_iden(),
                                    Alias::new("custom").into_iden(),
                                    Alias::new("doc_en").into_iden(),
                                    Alias::new("doc_fr").into_iden(),
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
                            .name("demo_page_category_id_demo_category_fkey")
                            .from(Alias::new("demo_page"), Alias::new("category_id"))
                            .to(Alias::new("demo_category"), Alias::new("id"))
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
                            .and_where(Expr::col(Alias::new("TABLE_NAME")).eq("demo_page"))
                            .and_where(Expr::col(Alias::new("COLUMN_NAME")).eq("category_id"))
                            .and_where(Expr::col(Alias::new("REFERENCED_TABLE_NAME")).is_not_null())
                            .to_owned(),
                    )
                    .await?
                    .is_some();
            if !backs_fk {
                manager
                    .create_index(
                        Index::create()
                            .name("idx_demo_page_category_id")
                            .table(Alias::new("demo_page"))
                            .col(Alias::new("category_id"))
                            .to_owned(),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("demo_page")).to_owned())
            .await?;
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .drop_type(
                    sea_query::extension::postgres::Type::drop()
                        .if_exists()
                        .name(Alias::new("pagetype"))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
