use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Blog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Blog::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Blog::Title).string().not_null())
                    .col(ColumnDef::new(Blog::Email).string().not_null())
                    .col(ColumnDef::new(Blog::Website).string())
                    .col(ColumnDef::new(Blog::Summary).text().not_null())
                    .col(ColumnDef::new(Blog::Content).text().not_null())
                    .col(
                        ColumnDef::new(Blog::CreatedAt)
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
            .drop_table(Table::drop().table(Blog::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Blog {
    Table,
    Id,
    Title,
    Email,
    Website,
    Summary,
    Content,
    CreatedAt,
}