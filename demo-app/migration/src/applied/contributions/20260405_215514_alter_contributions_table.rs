use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "ALTER TYPE ContributionType ADD VALUE IF NOT EXISTS 'Runique'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE ContributionType ADD VALUE IF NOT EXISTS 'Cours'"
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // WARNING: supprimer 'Runique' de ContributionType — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Cours' de ContributionType — migration manuelle requise (PG ne supporte pas DROP VALUE).
        Ok(())
    }
}
