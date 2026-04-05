use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "ALTER TYPE ChangelogCategory ADD VALUE IF NOT EXISTS 'Feature'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE ChangelogCategory ADD VALUE IF NOT EXISTS 'Ajouté'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE ChangelogCategory ADD VALUE IF NOT EXISTS 'Fix'"
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // WARNING: supprimer 'Feature' de ChangelogCategory — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Ajouté' de ChangelogCategory — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Fix' de ChangelogCategory — migration manuelle requise (PG ne supporte pas DROP VALUE).
        Ok(())
    }
}
