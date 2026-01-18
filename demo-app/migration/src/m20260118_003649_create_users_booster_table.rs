use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UsersBooster::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UsersBooster::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UsersBooster::Username).string().not_null())
                    .col(ColumnDef::new(UsersBooster::Email).string().not_null())
                    .col(ColumnDef::new(UsersBooster::Password).string().not_null())
                    .col(ColumnDef::new(UsersBooster::Bio).string())
                    .col(ColumnDef::new(UsersBooster::Website).string())
                    .col(
                        ColumnDef::new(UsersBooster::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(UsersBooster::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(UsersBooster::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UsersBooster::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UsersBooster {
    Table,
    Id,
    Username,
    Email,
    Password,
    Bio,
    Website,
    IsActive,
    CreatedAt,
    UpdatedAt,
}