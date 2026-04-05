use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Demarrage'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Database'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Security'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Web'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Admin'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Autres'"
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // WARNING: supprimer 'Demarrage' de SectionTheme — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Database' de SectionTheme — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Security' de SectionTheme — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Web' de SectionTheme — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Admin' de SectionTheme — migration manuelle requise (PG ne supporte pas DROP VALUE).

        // WARNING: supprimer 'Autres' de SectionTheme — migration manuelle requise (PG ne supporte pas DROP VALUE).
        Ok(())
    }
}
