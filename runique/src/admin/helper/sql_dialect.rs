//! Per-backend SQL fragments that sea-query does not translate on its own.
use crate::utils::aliases::ADb;
use sea_query::{Alias, Expr, ExprTrait, Func};

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
    match db.get_database_backend() {
        sea_orm::DbBackend::MySql => "CHAR",
        _ => "TEXT",
    }
}

/// `col` (cast to text for the real backend) compared equal to `val` — the
/// common pattern behind every builtin resource's column filter/scope check.
pub fn text_eq(db: &ADb, col: &str, val: &str) -> Expr {
    Expr::col(Alias::new(col))
        .cast_as(Alias::new(text_cast_type(db)))
        .eq(val)
}

/// Case-insensitive `LIKE` on `col` (cast to text for the real backend) against
/// `pattern` — the common pattern behind every builtin resource's search.
pub fn ilike(db: &ADb, col: &str, pattern: &str) -> Expr {
    Expr::expr(Func::lower(
        Expr::col(Alias::new(col)).cast_as(Alias::new(text_cast_type(db))),
    ))
    .like(pattern)
}
