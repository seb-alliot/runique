use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("blog"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(ColumnDef::new(Alias::new("email")).string().not_null())
                    .col(ColumnDef::new(Alias::new("website")).string().null())
                    .col(ColumnDef::new(Alias::new("summary")).text().not_null())
                    .col(ColumnDef::new(Alias::new("content")).text().not_null())
                    .col(ColumnDef::new_with_type(Alias::new("status"), ColumnType::Enum { name: Alias::new("BlogStatus").into_iden(), variants: vec![Alias::new("Draft").into_iden(), Alias::new("Published").into_iden(), Alias::new("Archived").into_iden()] }).not_null())
                    .col(ColumnDef::new(Alias::new("view_count")).integer().null().default(0))
                    .to_owned()
            )
            .await?;

        Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop()
                .table(Alias::new("blog"))
                .to_owned())
            .await?;
        Ok(())
}
}
