use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("test_relation_parent_tag"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("test_relation_parent_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("test_relation_tag_id")).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("test_relation_parent_tag_test_relation_parent_id_test_relation_parent_fkey")
                            .from(Alias::new("test_relation_parent_tag"), Alias::new("test_relation_parent_id"))
                            .to(Alias::new("test_relation_parent"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("test_relation_parent_tag_test_relation_tag_id_test_relation_tag_fkey")
                            .from(Alias::new("test_relation_parent_tag"), Alias::new("test_relation_tag_id"))
                            .to(Alias::new("test_relation_tag"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction)
                    )
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("test_relation_parent_tag_test_relation_parent_id_test_relation_tag_id_uniq")
                    .table(Alias::new("test_relation_parent_tag"))
                    .col(Alias::new("test_relation_parent_id"))
                    .col(Alias::new("test_relation_tag_id"))
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("test_relation_parent_tag_test_relation_parent_id_test_relation_tag_id_uniq").table(Alias::new("test_relation_parent_tag")).to_owned())
            .await?;

        manager
            .drop_table(Table::drop()
                .table(Alias::new("test_relation_parent_tag"))
                .to_owned())
            .await?;
        Ok(())
}
}
