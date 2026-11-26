//! PostgreSQL and MySQL: a foreign key is added or removed in place.
use sea_orm::DbErr;
use sea_orm_migration::SchemaManager;
use sea_query::{Alias, ForeignKey, ForeignKeyCreateStatement};

pub(crate) async fn drop_foreign_key(
    manager: &SchemaManager<'_>,
    table: &str,
    name: &str,
) -> Result<(), DbErr> {
    manager
        .drop_foreign_key(
            ForeignKey::drop()
                .table(Alias::new(table))
                .name(name)
                .to_owned(),
        )
        .await
}

pub(crate) async fn add_foreign_key(
    manager: &SchemaManager<'_>,
    key: ForeignKeyCreateStatement,
) -> Result<(), DbErr> {
    manager.create_foreign_key(key).await
}
