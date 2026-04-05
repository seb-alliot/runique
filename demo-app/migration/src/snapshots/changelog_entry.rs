use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("changelog_entry"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("version")).string().not_null())
                    .col(ColumnDef::new(Alias::new("release_date")).string().not_null())
                    .col(ColumnDef::new_with_type(Alias::new("category"), ColumnType::Enum { name: Alias::new("ChangelogCategory").into_iden(), variants: vec![Alias::new("Fix").into_iden(), Alias::new("Feature").into_iden(), Alias::new("Ajouté").into_iden()] }).not_null())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(ColumnDef::new(Alias::new("description")).string().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).integer().not_null())
                    .to_owned()
            )
            .await?;

        Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop()
                .table(Alias::new("changelog_entry"))
                .to_owned())
            .await?;
        Ok(())
}
}
