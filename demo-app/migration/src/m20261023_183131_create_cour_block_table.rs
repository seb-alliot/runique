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
                        .and_where(Expr::col(Alias::new("typname")).eq("courblocktype"))
                        .to_owned(),
                )
                .await?
                .is_some();
            if !exists {
                manager
                    .create_type(
                        sea_query::extension::postgres::Type::create()
                            .as_enum(Alias::new("courblocktype"))
                            .values([
                                Alias::new("code"),
                                Alias::new("text"),
                                Alias::new("table"),
                                Alias::new("list"),
                                Alias::new("warning"),
                            ])
                            .to_owned(),
                    )
                    .await?;
            }
        }

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("cour_block"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("chapitre_id"))
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("heading")).string().null())
                    .col(ColumnDef::new(Alias::new("content")).text().not_null())
                    .col(
                        ColumnDef::new_with_type(
                            Alias::new("block_type"),
                            ColumnType::Enum {
                                name: Alias::new("CourBlockType").into_iden(),
                                variants: vec![
                                    Alias::new("code").into_iden(),
                                    Alias::new("text").into_iden(),
                                    Alias::new("table").into_iden(),
                                    Alias::new("list").into_iden(),
                                    Alias::new("warning").into_iden(),
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
                            .name("cour_block_chapitre_id_chapitre_fkey")
                            .from(Alias::new("cour_block"), Alias::new("chapitre_id"))
                            .to(Alias::new("chapitre"), Alias::new("id"))
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
                            .and_where(Expr::col(Alias::new("TABLE_NAME")).eq("cour_block"))
                            .and_where(Expr::col(Alias::new("COLUMN_NAME")).eq("chapitre_id"))
                            .and_where(Expr::col(Alias::new("REFERENCED_TABLE_NAME")).is_not_null())
                            .to_owned(),
                    )
                    .await?
                    .is_some();
            if !backs_fk {
                manager
                    .create_index(
                        Index::create()
                            .name("idx_cour_block_chapitre_id")
                            .table(Alias::new("cour_block"))
                            .col(Alias::new("chapitre_id"))
                            .to_owned(),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("cour_block")).to_owned())
            .await?;
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .drop_type(
                    sea_query::extension::postgres::Type::drop()
                        .if_exists()
                        .name(Alias::new("courblocktype"))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
