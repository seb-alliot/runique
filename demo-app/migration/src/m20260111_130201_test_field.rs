// Migration pour créer la table de test des nouveaux fields

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TestFields::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TestFields::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    // Texte avancé
                    .col(ColumnDef::new(TestFields::Phone).string().not_null())
                    .col(ColumnDef::new(TestFields::Color).string().not_null())
                    .col(ColumnDef::new(TestFields::Uuid).string().not_null())
                    .col(ColumnDef::new(TestFields::Description).text().not_null())
                    .col(ColumnDef::new(TestFields::PostalCode).string().not_null())
                    // Numérique
                    .col(ColumnDef::new(TestFields::Price).double().not_null())
                    .col(ColumnDef::new(TestFields::Rating).integer().not_null())
                    .col(ColumnDef::new(TestFields::Quantity).integer().not_null())
                    .col(ColumnDef::new(TestFields::Discount).double().not_null())
                    .col(ColumnDef::new(TestFields::Amount).string().not_null())
                    // Temporel
                    .col(ColumnDef::new(TestFields::OpeningTime).string().not_null())
                    .col(
                        ColumnDef::new(TestFields::Duration)
                            .big_integer()
                            .not_null(),
                    )
                    // Fichiers
                    .col(ColumnDef::new(TestFields::ProfileImage).string().not_null())
                    .col(ColumnDef::new(TestFields::Attachments).text().not_null())
                    // Choix
                    .col(ColumnDef::new(TestFields::Preferences).text().not_null())
                    .col(ColumnDef::new(TestFields::Subscription).string().not_null())
                    // Meta
                    .col(
                        ColumnDef::new(TestFields::CreatedAt)
                            .timestamp()
                            .default(Expr::current_timestamp())
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TestFields::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TestFields {
    Table,
    Id,
    Phone,
    Color,
    Uuid,
    Description,
    PostalCode,
    Price,
    Rating,
    Quantity,
    Discount,
    Amount,
    OpeningTime,
    Duration,
    ProfileImage,
    Attachments,
    Preferences,
    Subscription,
    CreatedAt,
}
