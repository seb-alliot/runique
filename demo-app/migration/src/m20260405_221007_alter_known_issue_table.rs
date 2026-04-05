use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE IssueType ADD VALUE IF NOT EXISTS 'Ajoute'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE IssueType ADD VALUE IF NOT EXISTS 'Fix'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE IssueType ADD VALUE IF NOT EXISTS 'Manquant'")
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
