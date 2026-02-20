use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("test_all_fields"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("f_text")).string().null())
                    .col(ColumnDef::new(Alias::new("f_email")).string().null())
                    .col(ColumnDef::new(Alias::new("f_url")).string().null())
                    .col(ColumnDef::new(Alias::new("f_password")).string().null())
                    .col(ColumnDef::new(Alias::new("f_textarea")).text().null())
                    .col(ColumnDef::new(Alias::new("f_richtext")).text().null())
                    .col(ColumnDef::new(Alias::new("f_integer")).integer().null())
                    .col(ColumnDef::new(Alias::new("f_float")).string().null())
                    .col(ColumnDef::new(Alias::new("f_decimal")).string().null())
                    .col(ColumnDef::new(Alias::new("f_percent")).integer().null())
                    .col(ColumnDef::new(Alias::new("f_range")).integer().null())
                    .col(ColumnDef::new(Alias::new("f_checkbox")).boolean().null())
                    .col(ColumnDef::new(Alias::new("f_radio_single")).boolean().null())
                    .col(ColumnDef::new(Alias::new("f_select")).string().null())
                    .col(ColumnDef::new(Alias::new("f_select_multiple")).json().null())
                    .col(ColumnDef::new(Alias::new("f_radio_group")).string().null())
                    .col(ColumnDef::new(Alias::new("f_checkbox_group")).json().null())
                    .col(ColumnDef::new(Alias::new("f_date")).date_time().null())
                    .col(ColumnDef::new(Alias::new("f_time")).string().null())
                    .col(ColumnDef::new(Alias::new("f_datetime")).date_time().null())
                    .col(ColumnDef::new(Alias::new("f_duration")).big_integer().null())
                    .col(ColumnDef::new(Alias::new("f_file_image")).json().null())
                    .col(ColumnDef::new(Alias::new("f_file_document")).json().null())
                    .col(ColumnDef::new(Alias::new("f_file_any")).json().null())
                    .col(ColumnDef::new(Alias::new("f_color")).string().null())
                    .col(ColumnDef::new(Alias::new("f_slug")).string().null())
                    .col(ColumnDef::new(Alias::new("f_uuid")).uuid().null())
                    .col(ColumnDef::new(Alias::new("f_json")).json().null())
                    .col(ColumnDef::new(Alias::new("f_ip")).string().null())
                    .col(ColumnDef::new(Alias::new("created_at")).date_time().null())
                    .col(ColumnDef::new(Alias::new("updated_at")).date_time().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("test_all_fields")).to_owned())
            .await?;
        Ok(())
    }
}
