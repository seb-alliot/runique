use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("doc_section"))
                    .add_column(ColumnDef::new(Alias::new("theme")).string().null())
                    .to_owned(),
            )
            .await?;

        let db = manager.get_connection();
        db.execute_unprepared(
            "UPDATE doc_section SET theme = 'demarrage' WHERE slug IN ('installation', 'architecture', 'configuration', 'env');
             UPDATE doc_section SET theme = 'web'       WHERE slug IN ('routing', 'formulaire', 'flash', 'template');
             UPDATE doc_section SET theme = 'database'  WHERE slug IN ('orm', 'model');
             UPDATE doc_section SET theme = 'security'  WHERE slug IN ('middleware', 'auth', 'session');
             UPDATE doc_section SET theme = 'admin'     WHERE slug = 'admin';
             UPDATE doc_section SET theme = 'autres'    WHERE theme IS NULL;",
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Alias::new("doc_section"))
                    .drop_column(Alias::new("theme"))
                    .to_owned(),
            )
            .await
    }
}
