use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("cour_ia"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("context")).text().not_null())
                    .col(ColumnDef::new(Alias::new("contraintes")).text().not_null())
                    .col(ColumnDef::new(Alias::new("contrainte_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("cour_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).integer().not_null())
                    .to_owned()
            )
            .await?;

        Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop()
                .table(Alias::new("cour_ia"))
                .to_owned())
            .await?;
        Ok(())
}
}
