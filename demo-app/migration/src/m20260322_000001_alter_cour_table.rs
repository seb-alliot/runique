use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("cour"))
                    .add_column(
                        ColumnDef::new(Alias::new("difficulte"))
                            .string()
                            .not_null()
                            .default("debutant"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("cour"))
                    .drop_column(Alias::new("difficulte"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
