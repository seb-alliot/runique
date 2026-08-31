use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `demo_page.page_type` was created as a plain string (see
        // m20260413_133022_create_demo_page_table.rs) — this migration is where it
        // actually becomes an enum, so the type must be created here first. The
        // `ADD VALUE IF NOT EXISTS` calls below predate this fix and become no-ops
        // once the type already has all five values from creation; left in place
        // rather than removed, to keep this migration's diff to what was missing.
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN CREATE TYPE PageType AS ENUM ('code', 'form', 'custom', 'doc_en', 'doc_fr'); EXCEPTION WHEN duplicate_object THEN NULL; END $$"
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE PageType ADD VALUE IF NOT EXISTS 'doc_fr'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE PageType ADD VALUE IF NOT EXISTS 'custom'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE PageType ADD VALUE IF NOT EXISTS 'form'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE PageType ADD VALUE IF NOT EXISTS 'code'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE PageType ADD VALUE IF NOT EXISTS 'doc_en'")
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
