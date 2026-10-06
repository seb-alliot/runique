//! List fields (`checkbox` / `multichoice` in `model!{}`): filters on the
//! values chosen, stored in the field's own table (`{table}_{field}`).
use sea_orm::sea_query::Query;
use sea_orm::{ColumnTrait, Condition, EntityTrait, Iterable, PrimaryKeyToColumn, Value};
use std::marker::PhantomData;

/// Implemented by `model!{}` on each entity: `List` holds one [`ListField`]
/// per list field, the way `Column` holds one variant per column. It's how
/// `search!(Book => Genres has Genre::Roman)` finds a field with no column.
pub trait HasLists {
    type List;
}

/// A list field of the entity `O`: `C` is a column of its table, `V` the enum
/// its values come from — passing another enum doesn't compile.
pub struct ListField<O, C, V> {
    owner_id: C,
    value: C,
    _types: PhantomData<fn() -> (O, V)>,
}

impl<O, C, V> ListField<O, C, V>
where
    O: EntityTrait,
    C: ColumnTrait,
    V: Into<Value>,
{
    /// Used by `model!{}`: the `owner_id` and `value` columns of the field's table.
    pub const fn new(owner_id: C, value: C) -> Self {
        Self {
            owner_id,
            value,
            _types: PhantomData,
        }
    }

    /// The rows whose list contains `value`.
    pub fn has(&self, value: V) -> Condition {
        self.has_any([value])
    }

    /// The rows whose list contains at least one of `values`; none when
    /// `values` is empty.
    pub fn has_any(&self, values: impl IntoIterator<Item = V>) -> Condition {
        let owners = Query::select()
            .column(self.owner_id)
            .from(C::EntityName::default())
            .and_where(self.value.is_in(values))
            .to_owned();
        let pk = O::PrimaryKey::iter()
            .next()
            .expect("an entity has a primary key")
            .into_column();
        Condition::all().add(pk.in_subquery(owners))
    }

    /// The rows whose list contains every one of `values`; all rows when
    /// `values` is empty.
    pub fn has_all(&self, values: impl IntoIterator<Item = V>) -> Condition {
        values
            .into_iter()
            .fold(Condition::all(), |all, value| all.add(self.has(value)))
    }
}

/// Postgres only: the rows of `query` with the values of this list field, in
/// one round trip — each row carries its list, read by a subquery
/// (`string_agg`). Elsewhere, read the page then call `load_<field>()`.
#[cfg(feature = "postgres")]
impl<O, C, V> ListField<O, C, V>
where
    O: EntityTrait,
    O::Model: sea_orm::FromQueryResult,
    C: ColumnTrait,
    V: sea_orm::ActiveEnum + Into<Value>,
    <V as sea_orm::ActiveEnum>::Value: std::str::FromStr,
{
    pub async fn fetch_with<D>(
        &self,
        db: &D,
        query: sea_orm::Select<O>,
    ) -> Result<Vec<(O::Model, Vec<V>)>, sea_orm::DbErr>
    where
        D: sea_orm::ConnectionTrait,
    {
        use sea_orm::sea_query::Expr;
        use sea_orm::{
            DbBackend, DbErr, EntityName, FromQueryResult, IdenStatic, QuerySelect, QueryTrait,
        };

        if db.get_database_backend() != DbBackend::Postgres {
            return Err(DbErr::Custom(
                "ListField::fetch_with is Postgres-only: read the rows, then call load_<field>()"
                    .to_string(),
            ));
        }
        const ALIAS: &str = "runique_list_values";
        // Identifiers from `model!{}` (checked as SQL identifiers by runique_dsl),
        // never from a request. The separator is ASCII "unit separator": it can't
        // appear in an enum value.
        let owner = O::default();
        let pk = O::PrimaryKey::iter()
            .next()
            .expect("an entity has a primary key")
            .into_column();
        let list_table = C::EntityName::default();
        let values = Expr::cust(format!(
            r#"(SELECT string_agg("{value}"::text, chr(31) ORDER BY "id") FROM "{list}" WHERE "{list}"."{owner_id}" = "{table}"."{pk}")"#,
            value = self.value.as_str(),
            list = list_table.table_name(),
            owner_id = self.owner_id.as_str(),
            table = owner.table_name(),
            pk = pk.as_str(),
        ));
        let statement = query.column_as(values, ALIAS).build(DbBackend::Postgres);

        let mut rows = Vec::new();
        for row in db.query_all_raw(statement).await? {
            let model = O::Model::from_query_result(&row, "")?;
            let joined: Option<String> = row.try_get("", ALIAS)?;
            let mut list = Vec::new();
            for raw in joined.iter().flat_map(|j| j.split('\u{1f}')) {
                let stored = raw.parse::<V::Value>().map_err(|_| {
                    DbErr::Custom(format!("list value `{raw}` doesn't fit its enum"))
                })?;
                list.push(V::try_from_value(&stored)?);
            }
            rows.push((model, list));
        }
        Ok(rows)
    }
}
