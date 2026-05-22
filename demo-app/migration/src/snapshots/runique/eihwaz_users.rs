use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("eihwaz_users"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("bio")).text().null())
                    .col(ColumnDef::new(Alias::new("avatar")).string().null())
                    .col(ColumnDef::new(Alias::new("website")).string().null())
                    .col(ColumnDef::new(Alias::new("phone")).string().null())
                    .col(ColumnDef::new(Alias::new("birth_date")).date().null())
                    .col(ColumnDef::new(Alias::new("is_verified")).boolean().null())
                    .to_owned()
            )
            .await?;

        Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop()
                .table(Alias::new("eihwaz_users"))
                .to_owned())
            .await?;
        Ok(())
}
}
