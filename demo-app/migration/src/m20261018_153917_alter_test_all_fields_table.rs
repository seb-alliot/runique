use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // WARNING: SQLite cannot ALTER a column's type/nullable/unique constraint
        // (sea-query panics on any `modify_column` there) — this change only applies on
        // Postgres/MySQL; on SQLite the column keeps its current definition unchanged.
        if manager.get_connection().get_database_backend() != sea_orm::DbBackend::Sqlite {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("test_all_fields"))
                        .modify_column(
                            ColumnDef::new(Alias::new("f_decimal"))
                                .decimal_len(10, 2)
                                // The column is still text from 2026-04-13: the
                                // 2026-09-23 type change only left a comment.
                                // Postgres needs the conversion spelled out.
                                // No `.null()`: sea-query 1.0.2 would put USING
                                // after its DROP NOT NULL (invalid SQL); the
                                // column is nullable already, TYPE keeps it so.
                                .using(
                                    Expr::col(Alias::new("f_decimal"))
                                        .cast_as(Alias::new("numeric(10, 2)")),
                                ),
                        )
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // WARNING: SQLite cannot ALTER a column's type/nullable/unique constraint
        // (sea-query panics on any `modify_column` there) — this change only applies on
        // Postgres/MySQL; on SQLite the column keeps its current definition unchanged.
        if manager.get_connection().get_database_backend() != sea_orm::DbBackend::Sqlite {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("test_all_fields"))
                        .modify_column(ColumnDef::new(Alias::new("f_decimal")).decimal().null())
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}
