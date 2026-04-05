use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "ALTER TYPE RoadmapStatus ADD VALUE IF NOT EXISTS 'Future'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE RoadmapStatus ADD VALUE IF NOT EXISTS 'Planned'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE RoadmapStatus ADD VALUE IF NOT EXISTS 'Active'"
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // WARNING: supprimer 'Future' de RoadmapStatus — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Planned' de RoadmapStatus — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Active' de RoadmapStatus — migration manuelle requise (PG ne supporte pas DROP VALUE).
        Ok(())
    }
}
