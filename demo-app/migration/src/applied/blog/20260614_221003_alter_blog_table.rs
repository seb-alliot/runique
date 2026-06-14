use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "DO $$ BEGIN CREATE TYPE BlogStatus AS ENUM ('Draft', 'Published', 'Archived'); EXCEPTION WHEN duplicate_object THEN NULL; END $$"
        ).await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("blog"))
                    .add_column(ColumnDef::new(Alias::new("view_count")).integer().null().default(0))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("blog"))
                    .add_column(ColumnDef::new_with_type(Alias::new("status"), ColumnType::Enum { name: Alias::new("BlogStatus").into_iden(), variants: vec![Alias::new("Draft").into_iden(), Alias::new("Published").into_iden(), Alias::new("Archived").into_iden()] }).not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("blog"))
                    .drop_column(Alias::new("view_count"))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("blog"))
                    .drop_column(Alias::new("status"))
                    .to_owned(),
            )
            .await?;

        manager.get_connection().execute_unprepared(
            "DROP TYPE IF EXISTS BlogStatus"
        ).await?;
        Ok(())
    }
}
