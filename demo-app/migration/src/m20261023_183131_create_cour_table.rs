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
                        .and_where(Expr::col(Alias::new("typname")).eq("courtheme"))
                        .to_owned(),
                )
                .await?
                .is_some();
            if !exists {
                manager
                    .create_type(
                        sea_query::extension::postgres::Type::create()
                            .as_enum(Alias::new("courtheme"))
                            .values([
                                Alias::new("Fondamentaux"),
                                Alias::new("Mémoire & sûreté"),
                                Alias::new("Indispensables"),
                                Alias::new("Avancé"),
                                Alias::new("Runique"),
                            ])
                            .to_owned(),
                    )
                    .await?;
            }
        }

        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            let exists = manager
                .get_connection()
                .query_one(
                    &Query::select()
                        .expr(Expr::val(1))
                        .from((Alias::new("pg_catalog"), Alias::new("pg_type")))
                        .and_where(Expr::col(Alias::new("typname")).eq("difficulte"))
                        .to_owned(),
                )
                .await?
                .is_some();
            if !exists {
                manager
                    .create_type(
                        sea_query::extension::postgres::Type::create()
                            .as_enum(Alias::new("difficulte"))
                            .values([
                                Alias::new("debutant"),
                                Alias::new("intermediaire"),
                                Alias::new("avance"),
                                Alias::new("specifique"),
                            ])
                            .to_owned(),
                    )
                    .await?;
            }
        }

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("cour"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("slug")).string().not_null())
                    .col(ColumnDef::new(Alias::new("lang")).string().not_null())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(
                        ColumnDef::new_with_type(
                            Alias::new("theme"),
                            ColumnType::Enum {
                                name: Alias::new("CourTheme").into_iden(),
                                variants: vec![
                                    Alias::new("Fondamentaux").into_iden(),
                                    Alias::new("Mémoire & sûreté").into_iden(),
                                    Alias::new("Indispensables").into_iden(),
                                    Alias::new("Avancé").into_iden(),
                                    Alias::new("Runique").into_iden(),
                                ],
                            },
                        )
                        .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            Alias::new("difficulte"),
                            ColumnType::Enum {
                                name: Alias::new("Difficulte").into_iden(),
                                variants: vec![
                                    Alias::new("debutant").into_iden(),
                                    Alias::new("intermediaire").into_iden(),
                                    Alias::new("avance").into_iden(),
                                    Alias::new("specifique").into_iden(),
                                ],
                            },
                        )
                        .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("ordre")).integer().not_null())
                    .col(
                        ColumnDef::new(Alias::new("sort_order"))
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("cour")).to_owned())
            .await?;
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .drop_type(
                    sea_query::extension::postgres::Type::drop()
                        .if_exists()
                        .name(Alias::new("courtheme"))
                        .to_owned(),
                )
                .await?;
        }

        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .drop_type(
                    sea_query::extension::postgres::Type::drop()
                        .if_exists()
                        .name(Alias::new("difficulte"))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
