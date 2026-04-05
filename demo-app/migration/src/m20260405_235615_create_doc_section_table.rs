use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.get_connection().execute_unprepared(
            "CREATE TYPE SectionTheme AS ENUM ('Demarrage', 'Web', 'Database', 'Security', 'Admin', 'Autres')"
        ).await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("doc_section"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("slug")).string().not_null())
                    .col(ColumnDef::new(Alias::new("lang")).string().not_null())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(
                        ColumnDef::new(Alias::new("sort_order"))
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            Alias::new("theme"),
                            ColumnType::Enum {
                                name: Alias::new("SectionTheme").into_iden(),
                                variants: vec![
                                    Alias::new("Demarrage").into_iden(),
                                    Alias::new("Web").into_iden(),
                                    Alias::new("Database").into_iden(),
                                    Alias::new("Security").into_iden(),
                                    Alias::new("Admin").into_iden(),
                                    Alias::new("Autres").into_iden(),
                                ],
                            },
                        )
                        .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("doc_section")).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS SectionTheme")
            .await?;

        Ok(())
    }
}
