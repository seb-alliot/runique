use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("test_relation_profile"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("parent_id")).integer().not_null().unique_key())
                    .col(ColumnDef::new(Alias::new("bio")).string().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("test_relation_profile_parent_id_test_relation_parent_fkey")
                            .from(Alias::new("test_relation_profile"), Alias::new("parent_id"))
                            .to(Alias::new("test_relation_parent"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction)
                    )
                    .to_owned()
            )
            .await?;

        Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop()
                .table(Alias::new("test_relation_profile"))
                .to_owned())
            .await?;
        Ok(())
}
}
