use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `cour.theme`/`cour.difficulte` were created as plain strings (see
        // m20260413_133022_create_cour_table.rs) — this migration is where they
        // actually become enums. `CREATE TYPE` is Postgres-only syntax, guarded at
        // runtime rather than baked in at generation time (a migration file is fixed
        // forever once committed, so it must stay portable across engines).
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared(
                    "DO $$ BEGIN CREATE TYPE CourTheme AS ENUM ('Fondamentaux', 'Mémoire & sûreté', 'Indispensables', 'Avancé', 'Runique'); EXCEPTION WHEN duplicate_object THEN NULL; END $$"
                )
                .await?;

            manager
                .get_connection()
                .execute_unprepared(
                    "DO $$ BEGIN CREATE TYPE Difficulte AS ENUM ('debutant', 'intermediaire', 'avance', 'specifique'); EXCEPTION WHEN duplicate_object THEN NULL; END $$"
                )
                .await?;
        }

        // The actual column conversions — bare `modify_column` renders `ALTER COLUMN
        // ... TYPE enum_type` with no `USING` clause, which Postgres rejects on an
        // existing non-enum column. `.using()` is Postgres-specific (other backends'
        // renderers never read it), so setting it unconditionally is harmless there.
        // No `.not_null()` here: sea-query renders nullable as a separate "ALTER
        // COLUMN ... SET NOT NULL" clause and appends `USING` at the very end of the
        // whole statement regardless of which clause it belongs to, producing invalid
        // "TYPE t, SET NOT NULL USING expr" SQL. The columns' nullability isn't
        // changing here, so it's simply left untouched.
        // Skipped on SQLite entirely: sea-query's SQLite backend `panic!`s on ANY
        // `modify_column` (not a SQL rejection — the statement builder itself refuses).
        // Harmless to skip there: SQLite has no native enum (`ColumnType::Enum` renders
        // as plain `enum_text`), so the columns already behave as free-form text.
        if manager.get_connection().get_database_backend() != sea_orm::DbBackend::Sqlite {
            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("cour"))
                        .modify_column(
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
                            .using(Expr::col(Alias::new("theme")).cast_as(Alias::new("CourTheme"))),
                        )
                        .to_owned(),
                )
                .await?;

            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("cour"))
                        .modify_column(
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
                            .using(Expr::col(Alias::new("difficulte")).cast_as(Alias::new("Difficulte"))),
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
                        .table(Alias::new("cour"))
                        .modify_column(
                            ColumnDef::new(Alias::new("theme"))
                                .string()
                                .using(Expr::col(Alias::new("theme")).cast_as(Alias::new("text"))),
                        )
                        .to_owned(),
                )
                .await?;

            manager
                .alter_table(
                    Table::alter()
                        .table(Alias::new("cour"))
                        .modify_column(
                            ColumnDef::new(Alias::new("difficulte"))
                                .string()
                                .using(Expr::col(Alias::new("difficulte")).cast_as(Alias::new("text"))),
                        )
                        .to_owned(),
                )
                .await?;
        }

        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared("DROP TYPE IF EXISTS CourTheme")
                .await?;

            manager
                .get_connection()
                .execute_unprepared("DROP TYPE IF EXISTS Difficulte")
                .await?;
        }

        Ok(())
    }
}
