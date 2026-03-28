use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("chapitre"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("cour_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("slug")).string().not_null())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(ColumnDef::new(Alias::new("lead")).string().null())
                    .col(ColumnDef::new(Alias::new("sort_order")).integer().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(Alias::new("chapitre"), Alias::new("cour_id"))
                    .to(Alias::new("cour"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .table(Alias::new("chapitre"))
                    .name("chapitre_cour_id_cour_fkey")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Alias::new("chapitre"))
                .to_owned())
            .await?;
        Ok(())
    }
}
