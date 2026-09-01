use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // `ALTER TYPE` is Postgres-only syntax — via the builder (escapes the value the
        // same way every other sea-query statement does, unlike a hand-written string).
        // Type name lowercased: the raw `CREATE TYPE ChangelogCategory` in
        // m20260413_133022_create_changelog_entry_table.rs is never quoted, so Postgres
        // folded it to lowercase at creation time — but `Type::alter()` always quotes
        // its type name, so it must be given that same lowercase form or it looks for
        // a different, non-existent `"ChangelogCategory"` identifier.
        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {
            manager
                .alter_type(
                    sea_query::extension::postgres::Type::alter()
                        .name(Alias::new("changelogcategory"))
                        .add_value(Alias::new("Sécurité"))
                        .if_not_exists()
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
