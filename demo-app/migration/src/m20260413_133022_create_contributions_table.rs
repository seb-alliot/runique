use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `CREATE TYPE` is Postgres-only syntax — `ColumnType::Enum` below needs no
        // such guard, sea-query already renders it correctly per backend on its own.
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager.get_connection().execute_unprepared(
                "DO $$ BEGIN CREATE TYPE ContributionType AS ENUM ('Runique', 'Cours'); EXCEPTION WHEN duplicate_object THEN NULL; END $$"
            ).await?;
        }

        // FK declared inline on the CREATE TABLE itself, not via a separate
        // `create_foreign_key` afterward: SQLite cannot add a FK constraint to an
        // existing table (`Sqlite does not support modification of foreign key
        // constraints to existing tables`) — it must be present at creation time.
        // Inline is valid on every engine, not just required on SQLite.
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
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
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

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // No separate `drop_foreign_key` — SQLite can't ALTER-drop one any more than
        // it can ALTER-add one, and dropping the table below already takes the FK
        // with it on every engine (nothing else needs it kept around beforehand).
        manager
            .drop_table(Table::drop().table(Alias::new("contributions")).to_owned())
            .await?;
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared("DROP TYPE IF EXISTS ContributionType")
                .await?;
        }

        Ok(())
    }
}
