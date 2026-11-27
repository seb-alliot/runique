//! Diff calculation between two [`ParsedSchema`]: what changed between the
//! snapshot of a table and its model. One function per case, laid out as
//! in `generate`: the action (`create`, `delete`, `modify`, `rename`), then
//! the object. What a function here finds, the function at the same place in
//! `generate` writes.
mod create;
mod delete;
mod extensions;
mod models;
mod modify;
mod rename;

pub use create::creation_of;
pub use delete::dropped_tables;
pub use extensions::plan_extend_changes;
pub use models::compute_main_changes;
pub use rename::effective_renames;

use crate::migration::helpers::schemas::db_columns;
use crate::migration::types::{
    Changes, EnumChange, EnumChanges, ParsedColumn, ParsedEnum, ParsedFk, ParsedSchema, TableChange,
};
use std::collections::HashMap;

/// The columns of a table that exist in the database, by name.
pub(crate) type Columns<'a> = HashMap<&'a str, &'a ParsedColumn>;

/// Calculates differences between two table schemas. The values of an enum
/// are not compared here: an enum is an object of its own ([`diff_enum`]),
/// whatever the number of columns that use it.
pub fn diff_schemas(previous: &ParsedSchema, current: &ParsedSchema) -> Changes {
    let prev_cols = by_name(previous);
    let curr_cols = by_name(current);

    // A rename is found first: what it names is neither added nor dropped.
    let renamed_columns = rename::column(&prev_cols, &curr_cols);

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
        renamed_columns,
    }
}

/// Calculates what changed in an enum: `previous` is its snapshot, `None`
/// for an enum the database doesn't have yet, which is then created with
/// the values of `current`.
pub fn diff_enum(previous: Option<&ParsedEnum>, current: &ParsedEnum) -> EnumChanges {
    let shared = EnumChanges {
        name: current.name.clone(),
        values: current.values.clone(),
        places: current.places.clone(),
        ..EnumChanges::default()
    };
    let Some(previous) = previous else {
        return EnumChanges {
            change: EnumChange::Created,
            ..shared
        };
    };
    // A rename is found first: what it names is neither added nor dropped.
    let renamed = rename::enum_values(&previous.values, &current.values);
    EnumChanges {
        added_values: create::enum_values(&previous.values, &current.values, &renamed),
        dropped_values: delete::enum_values(&previous.values, &current.values, &renamed),
        renamed_values: renamed,
        ..shared
    }
}

/// Calculates what changed in every enum of the models, `previous` being the
/// enums the database has: the entries of each enum that has something to
/// migrate, in the order of `current`. An enum the models no longer declare
/// is not in the result: its type goes with the last column that used it.
///
/// An enum is looked up in three steps. The database has it under its name:
/// its values are compared. It has it under another name, which no model
/// declares any more and whose columns are now this enum's: it is renamed,
/// then its values are compared with the former ones, as a second entry.
/// Otherwise it is new, and created.
pub fn diff_enums(previous: &[ParsedEnum], current: &[ParsedEnum]) -> Vec<EnumChanges> {
    let renamed = rename::enum_types(previous, current);
    let mut all = Vec::new();
    for declared in current {
        let former = renamed
            .iter()
            .find(|(_, name)| *name == declared.name)
            .map(|(former, _)| former);
        let known_as = former.unwrap_or(&declared.name);
        let snapshot = previous.iter().find(|known| known.name == *known_as);
        if let Some(former) = former {
            all.push(EnumChanges {
                name: declared.name.clone(),
                change: EnumChange::Renamed,
                renamed_from: Some(former.clone()),
                values: declared.values.clone(),
                places: declared.places.clone(),
                ..EnumChanges::default()
            });
        }
        let changes = diff_enum(snapshot, declared);
        if !changes.is_empty() {
            all.push(changes);
        }
    }
    all
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
