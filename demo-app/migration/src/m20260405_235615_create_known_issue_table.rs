use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("CREATE TYPE IssueType AS ENUM ('Manquant', 'Ajoute', 'Fix')")
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("known_issue"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("version")).string().not_null())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(
                        ColumnDef::new(Alias::new("description"))
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            Alias::new("issue_type"),
                            ColumnType::Enum {
                                name: Alias::new("IssueType").into_iden(),
                                variants: vec![
                                    Alias::new("Manquant").into_iden(),
                                    Alias::new("Ajoute").into_iden(),
                                    Alias::new("Fix").into_iden(),
                                ],
                            },
                        )
                        .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("sort_order"))
                            .integer()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("known_issue")).to_owned())
            .await?;
        manager
            .get_connection()
            .execute_unprepared("DROP TYPE IF EXISTS IssueType")
            .await?;

        Ok(())
    }
}
