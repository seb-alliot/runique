use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Autres'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Admin'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Demarrage'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Web'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Database'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE SectionTheme ADD VALUE IF NOT EXISTS 'Security'"
        ).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {

        Ok(())
    }
}
