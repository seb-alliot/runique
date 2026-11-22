//! PostgreSQL: enum types are objects of their own, shared between columns.
use sea_orm::{ConnectionTrait, DbErr};
use sea_orm_migration::SchemaManager;
use sea_query::extension::postgres::Type;
use sea_query::{Alias, Expr, ExprTrait, Func, Query};

/// Drops the enum type `name` unless a column still uses it. Never a cascade:
/// a type shared by several tables stays until its last column is gone. The
/// current schema only, and the array form too (`_name`).
pub(super) async fn drop_enum_type_if_unused(
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
                    Expr::col(Alias::new("udt_schema")).eq(Func::cust(Alias::new("current_schema"))),
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
