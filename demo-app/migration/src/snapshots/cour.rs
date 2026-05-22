use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("cour"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("slug")).string().not_null())
                    .col(ColumnDef::new(Alias::new("lang")).string().not_null())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(ColumnDef::new_with_type(Alias::new("theme"), ColumnType::Enum { name: Alias::new("CourTheme").into_iden(), variants: vec![Alias::new("Fondamentaux").into_iden(), Alias::new("Mémoire & sûreté").into_iden(), Alias::new("Indispensables").into_iden(), Alias::new("Avancé").into_iden(), Alias::new("Runique").into_iden()] }).not_null())
                    .col(ColumnDef::new_with_type(Alias::new("difficulte"), ColumnType::Enum { name: Alias::new("Difficulte").into_iden(), variants: vec![Alias::new("debutant").into_iden(), Alias::new("intermediaire").into_iden(), Alias::new("avance").into_iden(), Alias::new("specifique").into_iden()] }).not_null())
                    .col(ColumnDef::new(Alias::new("ordre")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).integer().not_null())
                    .to_owned()
            )
            .await?;

        Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop()
                .table(Alias::new("cour"))
                .to_owned())
            .await?;
        Ok(())
}
}
