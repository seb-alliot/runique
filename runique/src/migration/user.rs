use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RuniqueUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RuniqueUsers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(RuniqueUsers::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(RuniqueUsers::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(RuniqueUsers::Password).string().not_null())
                    .col(
                        ColumnDef::new(RuniqueUsers::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(RuniqueUsers::IsStaff)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(RuniqueUsers::IsSuperuser)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(RuniqueUsers::Roles).text().null())
                    .col(
                        ColumnDef::new(RuniqueUsers::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(RuniqueUsers::UpdatedAt)
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
            .drop_table(Table::drop().table(RuniqueUsers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum RuniqueUsers {
    Table,
    Id,
    Username,
    Email,
    Password,
    IsActive,
    IsStaff,
    IsSuperuser,
    Roles,
    CreatedAt,
    UpdatedAt,
}
