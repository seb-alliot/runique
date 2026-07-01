//! Foreign-key label resolution for admin views.
//!
//! Centralises the `FK id -> related label` lookup used to display a
//! human-readable value instead of a raw id. Consumed by the generated
//! `list_fn`/`get_fn` (via [`resolve_fk_labels_in_rows`]) and reusable on
//! any other admin view (detail, history) that holds FK ids as JSON.
use std::collections::{HashMap, HashSet};

use sea_orm::{
    ConnectionTrait,
    sea_query::{Alias, Expr, ExprTrait, Query},
};
use serde_json::Value;

/// Normalises an FK value to a unified `String` key.
///
/// Accepts integer ids (i32/i64) and string ids (UUID) so the same lookup
/// map works regardless of the related table's primary-key type.
#[must_use]
pub fn fk_key(value: &Value) -> Option<String> {
    value
        .as_i64()
        .map(|n| n.to_string())
        .or_else(|| value.as_str().map(str::to_string))
}

/// Fetches an `id -> label` map for `ids` from `fk_table`.
///
/// `CAST(id AS TEXT)` keeps the lookup engine-agnostic (i32, i64, UUID).
///
/// SQL-injection safe: `ids` are bound through `is_in`; `fk_table` and
/// `fk_col` are static identifiers emitted by the `derive_form` macro, never
/// user input. Returns an empty map on query error — this is a display-only
/// path and must never break the request.
pub async fn fetch_fk_label_map<C: ConnectionTrait>(
    db: &C,
    fk_table: &str,
    fk_col: &str,
    ids: &[String],
) -> HashMap<String, String> {
    if ids.is_empty() {
        return HashMap::new();
    }
    let stmt = Query::select()
        .expr(Expr::cust("CAST(id AS TEXT)"))
        .expr(Expr::col(Alias::new(fk_col)))
        .from(Alias::new(fk_table))
        .and_where(Expr::cust("CAST(id AS TEXT)").is_in(ids.to_vec()))
        .to_owned();
    db.query_all(&stmt)
        .await
        .unwrap_or_default()
        .iter()
        .filter_map(|row| {
            let id = row.try_get_by_index::<String>(0).ok()?;
            let label = row.try_get_by_index::<String>(1).ok()?;
            Some((id, label))
        })
        .collect()
}

/// Display-layer entry point: resolves FK ids to labels across `rows` using
/// the owned specs carried on `AdminResource::fk_display`. Thin borrow-adapter
/// over [`resolve_fk_labels_in_rows`].
pub async fn resolve_fk_labels<C: ConnectionTrait>(
    db: &C,
    rows: &mut [Value],
    fk_display: &[(String, String, String)],
) {
    if fk_display.is_empty() {
        return;
    }
    let specs: Vec<(&str, &str, &str)> = fk_display
        .iter()
        .map(|(col, table, label_col)| (col.as_str(), table.as_str(), label_col.as_str()))
        .collect();
    resolve_fk_labels_in_rows(db, rows, &specs).await;
}

/// Replaces FK ids with their related labels, in place, across `rows`.
///
/// `fk_cols` is a slice of `(column, fk_table, fk_col)`. One query per FK
/// column (ids batched via `is_in`), so a single-row view is just a slice of
/// length 1. Rows whose FK value has no match are left untouched (dangling id).
pub async fn resolve_fk_labels_in_rows<C: ConnectionTrait>(
    db: &C,
    rows: &mut [Value],
    fk_cols: &[(&str, &str, &str)],
) {
    for (col, fk_table, fk_col) in fk_cols {
        let ids: Vec<String> = rows
            .iter()
            .filter_map(|r| r.get(*col).and_then(fk_key))
            .collect::<HashSet<String>>()
            .into_iter()
            .collect();
        let label_map = fetch_fk_label_map(db, fk_table, fk_col, &ids).await;
        if label_map.is_empty() {
            continue;
        }
        for row in rows.iter_mut() {
            if let Some(key) = row.get(*col).and_then(fk_key)
                && let Some(label) = label_map.get(&key)
            {
                row[*col] = Value::String(label.clone());
            }
        }
    }
}
