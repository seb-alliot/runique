//! PostgreSQL: enum types are objects of their own, shared between columns.
use sea_orm::{ConnectionTrait, DbErr};
use sea_orm_migration::SchemaManager;
use sea_query::extension::postgres::Type;
use sea_query::{Alias, Expr, ExprTrait, Func, Query};

/// Drops the enum type `name` unless a column still uses it. Never a cascade:
/// a type shared by several tables stays until its last column is gone. The
/// current schema only, and the array form too (`_name`).
pub(crate) async fn drop_enum_type_if_unused(
    manager: &SchemaManager<'_>,
    name: &str,
) -> Result<(), DbErr> {
    // Lowercased as where the type is created.
    let name = name.to_lowercase();
    let used = manager
        .get_connection()
        .query_one(
            &Query::select()
                .expr(Expr::val(1))
                .from((Alias::new("information_schema"), Alias::new("columns")))
                .and_where(
                    Expr::col(Alias::new("udt_schema"))
                        .eq(Func::cust(Alias::new("current_schema"))),
                )
                .and_where(
                    Expr::col(Alias::new("udt_name")).is_in([name.clone(), format!("_{name}")]),
                )
                .limit(1)
                .to_owned(),
        )
        .await?
        .is_some();
    if used {
        return Ok(());
    }
    manager
        .drop_type(Type::drop().if_exists().name(Alias::new(name)).to_owned())
        .await
}

/// Creates the enum type `name` with `values`, unless a type of that name is
/// already there: PostgreSQL has no `CREATE TYPE IF NOT EXISTS`, so its
/// catalogue is asked first. Only a type the current schema path reaches
/// counts, as for a table.
pub(crate) async fn create_enum_type_if_missing(
    manager: &SchemaManager<'_>,
    name: &str,
    values: &[&str],
) -> Result<(), DbErr> {
    // Lowercased as everywhere the type is named.
    let name = name.to_lowercase();
    let exists = manager
        .get_connection()
        .query_one(
            &Query::select()
                .expr(Expr::val(1))
                .from((Alias::new("pg_catalog"), Alias::new("pg_type")))
                .and_where(Expr::col(Alias::new("typname")).eq(name.clone()))
                .and_where(
                    Func::cust(Alias::new("pg_type_is_visible"))
                        .arg(Expr::col(Alias::new("oid")))
                        .into(),
                )
                .limit(1)
                .to_owned(),
        )
        .await?
        .is_some();
    if exists {
        return Ok(());
    }
    manager
        .create_type(
            Type::create()
                .as_enum(Alias::new(name))
                .values(values.iter().map(|value| Alias::new(*value)))
                .to_owned(),
        )
        .await
}

/// Adds `value` to the enum type `name`, unless it has it already.
pub(crate) async fn add_enum_value(
    manager: &SchemaManager<'_>,
    name: &str,
    value: &str,
) -> Result<(), DbErr> {
    manager
        .alter_type(
            Type::alter()
                .name(Alias::new(name.to_lowercase()))
                .add_value(Alias::new(value))
                .if_not_exists()
                .to_owned(),
        )
        .await
}

/// Renames the value `from` of the enum type `name`. The type is shared:
/// every column that uses it sees the new name at once, rows and defaults
/// included, and nothing is written to the tables.
pub(crate) async fn rename_enum_value(
    manager: &SchemaManager<'_>,
    name: &str,
    from: &str,
    to: &str,
) -> Result<(), DbErr> {
    manager
        .alter_type(
            Type::alter()
                .name(Alias::new(name.to_lowercase()))
                .rename_value(Alias::new(from), Alias::new(to))
                .to_owned(),
        )
        .await
}

/// Renames the enum type `from` to `to`, in place: columns hold a type by
/// what it is, not by its name, and follow without being written to.
pub(crate) async fn rename_enum_type(
    manager: &SchemaManager<'_>,
    from: &str,
    to: &str,
) -> Result<(), DbErr> {
    manager
        .alter_type(
            Type::alter()
                .name(Alias::new(from.to_lowercase()))
                .rename_to(Alias::new(to.to_lowercase()))
                .to_owned(),
        )
        .await
}
