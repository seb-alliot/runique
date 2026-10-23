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
                        .and_where(Expr::col(Alias::new("typname")).eq("roadmapstatus"))
                        .to_owned(),
                )
                .await?
                .is_some();
            if !exists {
                manager
                    .create_type(
                        sea_query::extension::postgres::Type::create()
                            .as_enum(Alias::new("roadmapstatus"))
                            .values([
                                Alias::new("Active"),
                                Alias::new("Planned"),
                                Alias::new("Future"),
                            ])
                            .to_owned(),
                    )
                    .await?;
            }
        }

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("roadmap_entry"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            Alias::new("status"),
                            ColumnType::Enum {
                                name: Alias::new("RoadmapStatus").into_iden(),
                                variants: vec![
                                    Alias::new("Active").into_iden(),
                                    Alias::new("Planned").into_iden(),
                                    Alias::new("Future").into_iden(),
                                ],
                            },
                        )
                        .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(ColumnDef::new(Alias::new("link_url")).string().null())
                    .col(ColumnDef::new(Alias::new("link_label")).string().null())
                    .col(ColumnDef::new(Alias::new("link_url_2")).string().null())
                    .col(ColumnDef::new(Alias::new("link_label_2")).string().null())
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
            .drop_table(Table::drop().table(Alias::new("roadmap_entry")).to_owned())
            .await?;
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .drop_type(
                    sea_query::extension::postgres::Type::drop()
                        .if_exists()
                        .name(Alias::new("roadmapstatus"))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
