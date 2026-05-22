use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("test_all_fields"))
                    .modify_column(
                        ColumnDef::new(Alias::new("f_radio_single"))
                            .boolean()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("test_all_fields"))
                    .modify_column(
                        ColumnDef::new(Alias::new("f_checkbox"))
                            .boolean()
                            .not_null(),
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
                    .table(Alias::new("test_all_fields"))
                    .modify_column(
                        ColumnDef::new(Alias::new("f_radio_single"))
                            .boolean()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("test_all_fields"))
                    .modify_column(ColumnDef::new(Alias::new("f_checkbox")).boolean().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
