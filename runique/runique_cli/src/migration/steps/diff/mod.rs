//! Diff calculation between two [`ParsedSchema`]: what changed between the
//! snapshot of a table and its model. One function per case, laid out as
//! in `generate`: the action (`create`, `delete`, `modify`, `rename`), then
//! the object. What a function here finds, the function at the same place in
//! `generate` writes.
mod create;
mod delete;
mod modify;
mod rename;

use crate::migration::helpers::schemas::db_columns;
use crate::migration::types::{Changes, ParsedColumn, ParsedFk, ParsedSchema, TableChange};
use std::collections::HashMap;

/// The columns of a table that exist in the database, by name.
pub(crate) type Columns<'a> = HashMap<&'a str, &'a ParsedColumn>;

/// Calculates differences between two table schemas.
pub fn diff_schemas(previous: &ParsedSchema, current: &ParsedSchema) -> Changes {
    let prev_cols = by_name(previous);
    let curr_cols = by_name(current);

    // A rename is found first: what it names is neither added nor dropped.
    let renamed_columns = rename::column(&prev_cols, &curr_cols);
    let renamed_values = rename::enum_value(&prev_cols, &curr_cols);

    Changes {
        table_name: current.table_name.clone(),
        added_columns: create::column(&prev_cols, &curr_cols, &renamed_columns),
        dropped_columns: delete::column(previous, &curr_cols, &renamed_columns),
        modified_columns: modify::column(&prev_cols, &curr_cols, &renamed_columns),
        renames: Default::default(),
        added_fks: create::key(previous, current),
        dropped_fks: delete::key(previous, current),
        added_indexes: create::index(previous, current),
        dropped_indexes: delete::index(previous, current),
        table: TableChange::Altered,
        enum_value_adds: create::enum_value(&prev_cols, &curr_cols, &renamed_values),
        enum_value_drops: delete::enum_value(&prev_cols, &curr_cols, &renamed_values),
        enum_renames: renamed_values,
        renamed_columns,
    }
}

fn by_name(schema: &ParsedSchema) -> Columns<'_> {
    db_columns(schema)
        .into_iter()
        .map(|c| (c.name.as_str(), c))
        .collect()
}

/// What makes a key the same key: its column, its target and its actions. A
/// key whose action changes is another key, dropped and added again.
fn key_identity(fk: &ParsedFk) -> String {
    format!(
        "{}->{}:{}:{:?}:{:?}",
        fk.from_column, fk.to_table, fk.to_column, fk.on_delete, fk.on_update
    )
}

/// The columns that are a string enum in both schemas, as (name, as it was,
/// as it is): the ones whose values can be compared.
///
/// A column that wasn't an enum before (e.g. a plain string turned into one)
/// is a genuine type change, not a value added to an existing enum type.
/// Compared here, every current variant would look "added" and emit
/// `ALTER TYPE … ADD VALUE` on a type that was never created (`CREATE TYPE`
/// is only ever emitted for a column's FIRST appearance as an enum). It is
/// left to `modify::column`, which sees the column entering the enum.
fn enum_columns<'a>(
    prev_cols: &Columns<'a>,
    curr_cols: &Columns<'a>,
) -> Vec<(&'a str, &'a ParsedColumn, &'a ParsedColumn)> {
    curr_cols
        .iter()
        .filter(|(_, curr)| !curr.enum_string_values.is_empty())
        .filter_map(|(name, curr)| {
            let prev = prev_cols.get(name)?;
            (!prev.enum_string_values.is_empty()).then_some((*name, *prev, *curr))
        })
        .collect()
}
