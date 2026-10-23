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
                        .and_where(Expr::col(Alias::new("typname")).eq("blogstatus"))
                        .to_owned(),
                )
                .await?
                .is_some();
            if !exists {
                manager
                    .create_type(
                        sea_query::extension::postgres::Type::create()
                            .as_enum(Alias::new("blogstatus"))
                            .values([
                                Alias::new("Draft"),
                                Alias::new("Published"),
                                Alias::new("Archived"),
                            ])
                            .to_owned(),
                    )
                    .await?;
            }
        }

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("blog"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(
                        ColumnDef::new(Alias::new("email"))
                            .string_len(254)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("website")).string().null())
                    .col(ColumnDef::new(Alias::new("summary")).text().not_null())
                    .col(ColumnDef::new(Alias::new("content")).text().not_null())
                    .col(
                        ColumnDef::new_with_type(
                            Alias::new("status"),
                            ColumnType::Enum {
                                name: Alias::new("BlogStatus").into_iden(),
                                variants: vec![
                                    Alias::new("Draft").into_iden(),
                                    Alias::new("Published").into_iden(),
                                    Alias::new("Archived").into_iden(),
                                ],
                            },
                        )
                        .not_null()
                        .default("Draft"),
                    )
                    .col(
                        ColumnDef::new(Alias::new("view_count"))
                            .integer()
                            .null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("blog")).to_owned())
            .await?;
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .drop_type(
                    sea_query::extension::postgres::Type::drop()
                        .if_exists()
                        .name(Alias::new("blogstatus"))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
