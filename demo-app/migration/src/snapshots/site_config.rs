use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("site_config"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("key")).string().not_null())
                    .col(ColumnDef::new(Alias::new("value")).string().not_null())
                    .col(ColumnDef::new(Alias::new("description")).string().null())
                    .col(ColumnDef::new(Alias::new("is_public")).boolean().null().default(true))
                    .col(ColumnDef::new(Alias::new("sort_order")).integer().null().default(0))
                    .to_owned()
            )
            .await?;

        Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop()
                .table(Alias::new("site_config"))
                .to_owned())
            .await?;
        Ok(())
}
}
