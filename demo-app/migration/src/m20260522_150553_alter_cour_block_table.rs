use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `cour_block.block_type` was created as a plain string (see
        // m20260413_133022_create_cour_block_table.rs) — this migration is where it
        // actually becomes an enum. `CREATE TYPE` is Postgres-only syntax, guarded at
        // runtime rather than baked in at generation time (a migration file is fixed
        // forever once committed, so it must stay portable across engines).
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared(
                    "DO $$ BEGIN CREATE TYPE CourBlockType AS ENUM ('code', 'text', 'table', 'list', 'warning'); EXCEPTION WHEN duplicate_object THEN NULL; END $$"
                )
                .await?;
        }

        // The actual column conversion — bare `modify_column` renders `ALTER COLUMN
        // ... TYPE enum_type` with no `USING` clause, which Postgres rejects on an
        // existing non-enum column. `.using()` is Postgres-specific (other backends'
        // renderers never read it), so setting it unconditionally is harmless there.
        // No `.not_null()` here: sea-query renders nullable as a separate "ALTER
        // COLUMN ... SET NOT NULL" clause and appends `USING` at the very end of the
        // whole statement regardless of which clause it belongs to, producing invalid
        // "TYPE t, SET NOT NULL USING expr" SQL. The column's nullability isn't
        // changing here, so it's simply left untouched.
        // Skipped on SQLite entirely: sea-query's SQLite backend `panic!`s on ANY
        // `modify_column` (not a SQL rejection — the statement builder itself refuses).
        // Harmless to skip there: SQLite has no native enum (`ColumnType::Enum` renders
        // as plain `enum_text`), so the column already behaves as free-form text.
        if manager.get_connection().get_database_backend() != sea_orm::DbBackend::Sqlite {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("cour_block"))
                        .modify_column(
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
                            .using(
                                Expr::col(Alias::new("block_type"))
                                    .cast_as(Alias::new("CourBlockType")),
                            ),
                        )
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_connection().get_database_backend() != sea_orm::DbBackend::Sqlite {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("cour_block"))
                        .modify_column(
                            ColumnDef::new(Alias::new("block_type")).string().using(
                                Expr::col(Alias::new("block_type")).cast_as(Alias::new("text")),
                            ),
                        )
                        .to_owned(),
                )
                .await?;
        }

        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared("DROP TYPE IF EXISTS CourBlockType")
                .await?;
        }

        Ok(())
    }
}
