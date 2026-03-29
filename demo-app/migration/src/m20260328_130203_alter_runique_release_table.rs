use sea_orm::DbBackend;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SQLite ne supporte pas MODIFY COLUMN — la contrainte NOT NULL
        // est gérée au niveau applicatif pour ce backend.
        if manager.get_database_backend() == DbBackend::Sqlite {
            return Ok(());
        }

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("runique_release"))
                    .modify_column(ColumnDef::new(Alias::new("crates_url")).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("runique_release"))
                    .modify_column(ColumnDef::new(Alias::new("github_url")).string().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_database_backend() == DbBackend::Sqlite {
            return Ok(());
        }

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("runique_release"))
                    .modify_column(ColumnDef::new(Alias::new("crates_url")).string().null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("runique_release"))
                    .modify_column(ColumnDef::new(Alias::new("github_url")).string().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
