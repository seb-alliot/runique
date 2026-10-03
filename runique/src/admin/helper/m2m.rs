//! Many-to-many links of a resource, written by the generated admin code into
//! its junction table — with typed values, inside the caller's transaction,
//! and never an error left unreported.
use crate::utils::aliases::StrMap;
use sea_orm::{ConnectionTrait, DbErr};
use sea_query::{Alias, Expr, ExprTrait, Query, Value};

/// Writes the links of `owner` found in `data` (keys `{prefix}{target_id}`, as
/// the checkboxes of the form send them) into `junction`. With `replace`, the
/// links it had before go first — the edit of an existing row.
///
/// A target id is an integer or a UUID; any other key is not a link and is
/// skipped. Called with a transaction, so a failure leaves the links as they
/// were.
#[allow(clippy::too_many_arguments)]
pub async fn write_links<C: ConnectionTrait>(
    db: &C,
    junction: &str,
    self_fk: &str,
    target_fk: &str,
    owner: Value,
    data: &StrMap,
    prefix: &str,
    replace: bool,
) -> Result<(), DbErr> {
    if replace {
        db.execute(
            &Query::delete()
                .from_table(Alias::new(junction))
                .and_where(Expr::col(Alias::new(self_fk)).eq(owner.clone()))
                .to_owned(),
        )
        .await?;
    }
    for target in data
        .keys()
        .filter_map(|key| key.strip_prefix(prefix))
        .filter_map(target_value)
    {
        db.execute(
            &Query::insert()
                .into_table(Alias::new(junction))
                .columns([Alias::new(self_fk), Alias::new(target_fk)])
                .values_panic([Expr::val(owner.clone()), Expr::val(target)])
                .to_owned(),
        )
        .await?;
    }
    Ok(())
}

/// The id a checkbox key carries, typed for the junction column.
fn target_value(raw: &str) -> Option<Value> {
    raw.parse::<i64>()
        .map(Value::from)
        .ok()
        .or_else(|| uuid::Uuid::parse_str(raw).ok().map(Value::from))
}
