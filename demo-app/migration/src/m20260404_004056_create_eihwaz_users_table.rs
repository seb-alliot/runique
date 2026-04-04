use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("eihwaz_users"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("username")).string().not_null().unique_key())
                    .col(ColumnDef::new(Alias::new("email")).string().not_null().unique_key())
                    .col(ColumnDef::new(Alias::new("password")).string().not_null())
                    .col(ColumnDef::new(Alias::new("is_active")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("is_staff")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("is_superuser")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("roles")).string().null())
                    .col(ColumnDef::new(Alias::new("created_at")).date_time().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Alias::new("updated_at")).date_time().not_null().default(Expr::current_timestamp()))
                    .to_owned()
            )
            .await?;

        manager.get_connection().execute_unprepared(
            "CREATE OR REPLACE FUNCTION set_updated_at_eihwaz_users() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW(); RETURN NEW; END; $$ LANGUAGE plpgsql;"
        ).await?;
        manager.get_connection().execute_unprepared(
            "CREATE TRIGGER trg_eihwaz_users_updated_at BEFORE UPDATE ON eihwaz_users FOR EACH ROW EXECUTE PROCEDURE set_updated_at_eihwaz_users();"
        ).await?;

        Ok(())
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "DROP TRIGGER IF EXISTS trg_eihwaz_users_updated_at ON eihwaz_users;"
        ).await?;
        manager.get_connection().execute_unprepared(
            "DROP FUNCTION IF EXISTS set_updated_at_eihwaz_users();"
        ).await?;

        manager
            .drop_table(Table::drop()
                .table(Alias::new("eihwaz_users"))
                .to_owned())
            .await?;
        Ok(())
}
}
