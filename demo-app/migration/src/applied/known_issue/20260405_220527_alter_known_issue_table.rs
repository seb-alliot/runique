use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "ALTER TYPE IssueType ADD VALUE IF NOT EXISTS 'Ajoute'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE IssueType ADD VALUE IF NOT EXISTS 'Fix'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE IssueType ADD VALUE IF NOT EXISTS 'Manquant'"
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // WARNING: supprimer 'Ajoute' de IssueType — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Fix' de IssueType — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Manquant' de IssueType — migration manuelle requise (PG ne supporte pas DROP VALUE).
        Ok(())
    }
}
