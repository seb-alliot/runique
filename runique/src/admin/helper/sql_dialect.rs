//! Per-backend SQL fragments that sea-query does not translate on its own.
use crate::utils::aliases::ADb;
use sea_orm::{DbBackend, EntityTrait};
use sea_query::{Alias, Condition, Expr, ExprTrait, Func, LikeExpr};

/// Cast target type for comparing/reading a column as text, valid on the real
/// backend behind `db`.
///
/// `sea_query::Expr::cast_as`/`Expr::cust` take an arbitrary type-name string and
/// emit it verbatim — unlike `ColumnType::Enum`, which sea-query's own per-backend
/// renderer does translate automatically, a raw `CAST(x AS ...)` target name is
/// never dialect-aware. MariaDB/MySQL reject `TEXT` as a `CAST` target (`CAST(x AS
/// CHAR)` is the equivalent there); PostgreSQL and SQLite both accept `TEXT`.
/// `db.get_database_backend()` is sea-orm's own detection — this function only
/// supplies the one mapping sea-orm has no way to know on its own.
pub fn text_cast_type(db: &ADb) -> &'static str {
    text_cast_type_of(db.get_database_backend())
}

/// Same as [`text_cast_type`], from the backend alone — for code generic over
/// any `ConnectionTrait`.
pub fn text_cast_type_of(backend: sea_orm::DbBackend) -> &'static str {
    match backend {
        sea_orm::DbBackend::MySql => "CHAR",
        _ => "TEXT",
    }
}
/// Not `\`: MySQL treats it as an escape inside string literals too.
const LIKE_ESCAPE: char = '!';

/// `%` and `_` typed by the user are matched literally, not as wildcards.
fn escape_like(term: &str) -> String {
    let mut out = String::with_capacity(term.len());
    for c in term.chars() {
        if matches!(c, '%' | '_' | LIKE_ESCAPE) {
            out.push(LIKE_ESCAPE);
        }
        out.push(c);
    }
    out
}
/// Case-insensitive search for `term` in any of `cols`. `None` when there is
/// nothing to search (no column, or a blank term): the caller then applies no
/// condition at all.
pub fn search_condition<E: EntityTrait>(
    backend: DbBackend,
    cols: &[E::Column],
    term: &str,
) -> Option<Condition> {
    let term = term.trim();
    if cols.is_empty() || term.is_empty() {
        return None;
    }
    let pattern = format!("%{}%", escape_like(&term.to_lowercase()));
    let cast = Alias::new(text_cast_type_of(backend));
    Some(cols.iter().fold(Condition::any(), |cond, col| {
        cond.add(
            Expr::expr(Func::lower(
                Expr::col((E::default(), *col)).cast_as(cast.clone()),
            ))
            .like(LikeExpr::new(pattern.clone()).escape(LIKE_ESCAPE)),
        )
    }))
}

/// `col` (cast to text for the real backend) compared equal to `val` — the
/// common pattern behind every builtin resource's column filter/scope check.
pub fn text_eq(db: &ADb, col: &str, val: &str) -> Expr {
    Expr::col(Alias::new(col))
        .cast_as(Alias::new(text_cast_type(db)))
        .eq(val)
}

/// Case-insensitive equality (`LOWER(col) = LOWER(val)`), both sides lowered
/// by the database itself so the folding rules always match. SQLite's `LOWER`
/// only folds ASCII: `Œufs` and `œufs` stay distinct there.
pub fn text_eq_ci(db: &ADb, col: &str, val: &str) -> Expr {
    Expr::expr(Func::lower(
        Expr::col(Alias::new(col)).cast_as(Alias::new(text_cast_type(db))),
    ))
    .eq(Func::lower(Expr::val(val)))
}

/// Case-insensitive `LIKE` on `col` (cast to text for the real backend) against
/// `pattern` — the common pattern behind every builtin resource's search.
pub fn ilike(db: &ADb, col: &str, pattern: &str) -> Expr {
    Expr::expr(Func::lower(
        Expr::col(Alias::new(col)).cast_as(Alias::new(text_cast_type(db))),
    ))
    .like(pattern)
}

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::search_condition;
    use crate::admin::refonte_admin::gate::test_support::fixtures::note::{Column, Entity};
    use sea_orm::{DbBackend, EntityTrait, QueryFilter, QueryTrait};

    fn sql(backend: DbBackend, cols: &[Column], term: &str) -> String {
        let cond = search_condition::<Entity>(backend, cols, term).unwrap();
        Entity::find().filter(cond).build(backend).to_string()
    }

    #[test]
    fn the_term_is_lowered_and_wrapped() {
        let sql = sql(DbBackend::Sqlite, &[Column::OwnerId], "ABC");
        assert!(sql.contains(r#"LOWER(CAST("notes"."owner_id" AS"#), "{sql}");
        assert!(sql.contains("LIKE '%abc%' ESCAPE '!'"), "{sql}");
    }

    #[test]
    fn wildcards_typed_by_the_user_are_escaped() {
        let sql = sql(DbBackend::Sqlite, &[Column::OwnerId], "5_0%!");
        assert!(sql.contains("'%5!_0!%!!%'"), "{sql}");
    }

    #[test]
    fn every_column_is_searched_with_or() {
        let sql = sql(DbBackend::Sqlite, &[Column::Id, Column::OwnerId], "a");
        assert!(sql.contains(r#""notes"."id""#), "{sql}");
        assert!(sql.contains(" OR "), "{sql}");
    }

    #[test]
    fn the_cast_follows_the_backend() {
        assert!(sql(DbBackend::MySql, &[Column::Id], "a").contains("CHAR)"));
        assert!(sql(DbBackend::Postgres, &[Column::Id], "a").contains("TEXT)"));
    }

    #[test]
    fn no_column_or_a_blank_term_is_no_search() {
        assert!(search_condition::<Entity>(DbBackend::Sqlite, &[], "a").is_none());
        assert!(search_condition::<Entity>(DbBackend::Sqlite, &[Column::Id], "   ").is_none());
    }
}
