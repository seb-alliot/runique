use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "ALTER TYPE CourTheme ADD VALUE IF NOT EXISTS 'Avancé'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE CourTheme ADD VALUE IF NOT EXISTS 'Mémoire & sûreté'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE CourTheme ADD VALUE IF NOT EXISTS 'Fondamentaux'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE CourTheme ADD VALUE IF NOT EXISTS 'Indispensables'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE CourTheme ADD VALUE IF NOT EXISTS 'Runique'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE Difficulte ADD VALUE IF NOT EXISTS 'specifique'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE Difficulte ADD VALUE IF NOT EXISTS 'intermediaire'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE Difficulte ADD VALUE IF NOT EXISTS 'debutant'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE Difficulte ADD VALUE IF NOT EXISTS 'avance'"
        ).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {

        Ok(())
    }
}
