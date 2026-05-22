use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "ALTER TYPE CourBlockType ADD VALUE IF NOT EXISTS 'list'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE CourBlockType ADD VALUE IF NOT EXISTS 'table'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE CourBlockType ADD VALUE IF NOT EXISTS 'warning'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE CourBlockType ADD VALUE IF NOT EXISTS 'code'"
        ).await?;

        manager.get_connection().execute_unprepared(
            "ALTER TYPE CourBlockType ADD VALUE IF NOT EXISTS 'text'"
        ).await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {

        Ok(())
    }
}
