//! MySQL and SQLite: an enum column is text, and the enum itself only exists
//! in the code of the project. A value is nothing but what the rows hold.
use sea_orm::{ConnectionTrait, DbErr};
use sea_orm_migration::SchemaManager;
use sea_query::{Alias, Expr, ExprTrait, Query};

/// Gives the rows of each of `places` (table, column) that hold `from` the
/// value `to`. The other rows are left as they are.
pub(crate) async fn rename_enum_value(
    manager: &SchemaManager<'_>,
    places: &[(&str, &str)],
    from: &str,
    to: &str,
) -> Result<(), DbErr> {
    for (table, column) in places {
        manager
            .get_connection()
            .execute(
                &Query::update()
                    .table(Alias::new(*table))
                    .value(Alias::new(*column), to)
                    .and_where(Expr::col(Alias::new(*column)).eq(from))
                    .to_owned(),
            )
            .await?;
    }
    Ok(())
}
