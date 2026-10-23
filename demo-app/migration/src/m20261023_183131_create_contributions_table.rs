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
                        .and_where(Expr::col(Alias::new("typname")).eq("contributiontype"))
                        .to_owned(),
                )
                .await?
                .is_some();
            if !exists {
                manager
                    .create_type(
                        sea_query::extension::postgres::Type::create()
                            .as_enum(Alias::new("contributiontype"))
                            .values([Alias::new("Runique"), Alias::new("Cours")])
                            .to_owned(),
                    )
                    .await?;
            }
        }

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("contributions"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
                    .col(
                        ColumnDef::new_with_type(
                            Alias::new("contribution_type"),
                            ColumnType::Enum {
                                name: Alias::new("ContributionType").into_iden(),
                                variants: vec![
                                    Alias::new("Runique").into_iden(),
                                    Alias::new("Cours").into_iden(),
                                ],
                            },
                        )
                        .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("title"))
                            .string_len(200)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("content")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("contributions_user_id_eihwaz_users_fkey")
                            .from(Alias::new("contributions"), Alias::new("user_id"))
                            .to(Alias::new("eihwaz_users"), Alias::new("id"))
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
                            .and_where(Expr::col(Alias::new("TABLE_NAME")).eq("contributions"))
                            .and_where(Expr::col(Alias::new("COLUMN_NAME")).eq("user_id"))
                            .and_where(Expr::col(Alias::new("REFERENCED_TABLE_NAME")).is_not_null())
                            .to_owned(),
                    )
                    .await?
                    .is_some();
            if !backs_fk {
                manager
                    .create_index(
                        Index::create()
                            .name("idx_contributions_user_id")
                            .table(Alias::new("contributions"))
                            .col(Alias::new("user_id"))
                            .to_owned(),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("contributions")).to_owned())
            .await?;
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .drop_type(
                    sea_query::extension::postgres::Type::drop()
                        .if_exists()
                        .name(Alias::new("contributiontype"))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
