use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("users_booster"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("username")).string().not_null())
                    .col(ColumnDef::new(Alias::new("email")).string().not_null())
                    .col(ColumnDef::new(Alias::new("password")).string().not_null())
                    .col(ColumnDef::new(Alias::new("bio")).string().null())
                    .col(ColumnDef::new(Alias::new("website")).string().null())
                    .col(ColumnDef::new(Alias::new("is_active")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("created_at")).date_time().null())
                    .col(ColumnDef::new(Alias::new("updated_at")).date_time().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("users_booster")).to_owned())
            .await?;
        Ok(())
    }
}
