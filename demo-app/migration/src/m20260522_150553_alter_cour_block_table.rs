use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `cour_block.block_type` was created as a plain string (see
        // m20260413_133022_create_cour_block_table.rs) — this migration is where it
        // actually becomes an enum, so the type must be created here first. The
        // `ADD VALUE IF NOT EXISTS` calls below predate this fix and become no-ops
        // once the type already has all five values from creation; left in place
        // rather than removed, to keep this migration's diff to what was missing.
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN CREATE TYPE CourBlockType AS ENUM ('code', 'text', 'table', 'list', 'warning'); EXCEPTION WHEN duplicate_object THEN NULL; END $$"
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE CourBlockType ADD VALUE IF NOT EXISTS 'list'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE CourBlockType ADD VALUE IF NOT EXISTS 'table'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE CourBlockType ADD VALUE IF NOT EXISTS 'warning'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE CourBlockType ADD VALUE IF NOT EXISTS 'code'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE CourBlockType ADD VALUE IF NOT EXISTS 'text'")
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
