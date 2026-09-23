use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // WARNING: type change on column 'f_decimal': String -> Decimal
        // Manual migration required.

        manager
            .create_index(
                Index::create()
                    .name("test_all_fields_f_text_f_integer_uniq")
                    .table(Alias::new("test_all_fields"))
                    .col(Alias::new("f_text"))
                    .col(Alias::new("f_integer"))
                    .unique()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("test_all_fields_f_text_f_integer_uniq")
                    .table(Alias::new("test_all_fields"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
