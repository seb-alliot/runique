use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `cour.theme`/`cour.difficulte` were created as plain strings (see
        // m20260413_133022_create_cour_table.rs) — this migration is where they
        // actually become enums, so both types must be created here first. The
        // `ADD VALUE IF NOT EXISTS` calls below predate this fix and become no-ops
        // once each type already has all its values from creation; left in place
        // rather than removed, to keep this migration's diff to what was missing.
        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN CREATE TYPE CourTheme AS ENUM ('Fondamentaux', 'Mémoire & sûreté', 'Indispensables', 'Avancé', 'Runique'); EXCEPTION WHEN duplicate_object THEN NULL; END $$"
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "DO $$ BEGIN CREATE TYPE Difficulte AS ENUM ('debutant', 'intermediaire', 'avance', 'specifique'); EXCEPTION WHEN duplicate_object THEN NULL; END $$"
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE CourTheme ADD VALUE IF NOT EXISTS 'Avancé'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE CourTheme ADD VALUE IF NOT EXISTS 'Mémoire & sûreté'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE CourTheme ADD VALUE IF NOT EXISTS 'Fondamentaux'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE CourTheme ADD VALUE IF NOT EXISTS 'Indispensables'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE CourTheme ADD VALUE IF NOT EXISTS 'Runique'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE Difficulte ADD VALUE IF NOT EXISTS 'specifique'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE Difficulte ADD VALUE IF NOT EXISTS 'intermediaire'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE Difficulte ADD VALUE IF NOT EXISTS 'debutant'")
            .await?;

        manager
            .get_connection()
            .execute_unprepared("ALTER TYPE Difficulte ADD VALUE IF NOT EXISTS 'avance'")
            .await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
