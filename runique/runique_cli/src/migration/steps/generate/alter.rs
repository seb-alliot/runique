//! The migration of a table that changes: the cases its change asks for.
use super::{builder::BuilderPlan, constraints::fk_constraint_name, instruction::File};
use crate::migration::types::Changes;

/// True if `body` has a line that isn't blank or a `//` comment — i.e. it actually calls `manager`.
/// A type-change diff only emits a `// Manual migration required.` comment, which is non-empty
/// text but no real statement, so a plain `is_empty()` check picks `manager` and leaves it unused.
pub(super) fn body_has_code(body: &str) -> bool {
    body.lines()
        .any(|line| !line.trim().is_empty() && !line.trim_start().starts_with("//"))
}

/// The cases a change of table asks for, in any order: where each runs and
/// its way back are the builder's to tell.
pub(super) fn alter_plan(change: &Changes) -> BuilderPlan<'_> {
    let mut plan = BuilderPlan::for_every_engine();
    let table = change.table_name.as_str();
    // The name the table had when its keys and indexes were named.
    let former = change.renames.from.as_deref().unwrap_or(table);

    // The table takes its new name, then the indexes and keys named after
    // the former one take theirs.
    if let Some(old) = &change.renames.from {
        plan.rename_table(old, table);
    }
    for (old, new) in &change.renames.indexes {
        plan.rename_index(table, (old, former), (new, table));
    }
    for key in &change.renames.keys {
        plan.rename_key(
            table,
            (key.old_table.as_str(), key.old_target.as_str()),
            (table, key.key.to_table.as_str()),
            &key.key,
            &key.target_at_run,
        );
    }
    for (old, new) in &change.renamed_columns {
        plan.rename_column(table, old, new);
    }

    // What is in the way of the columns goes first. A key of a renamed
    // table that goes still has the name it was given under the former
    // table name, and so does its index.
    for index in &change.dropped_indexes {
        plan.delete_index(table, former, index, File::Migration);
    }
    for key in &change.dropped_fks {
        plan.delete_key(
            table,
            fk_constraint_name(former, &key.from_column, &key.to_table),
            key,
            File::Migration,
        );
    }

    for (old, new) in &change.modified_columns {
        // Becoming (or stopping being) an enum: `col_type` alone doesn't capture this
        // (an enum column keeps col_type "String"), so this must be checked before the
        // generic "type change => manual" fallback below, which would otherwise catch
        // ordinary type changes but silently miss this one entirely.
        if old.enum_string_values.is_empty() != new.enum_string_values.is_empty() {
            plan.modify_enum_column(table, old, new);
            continue;
        }

        // type change => manual
        if old.col_type != new.col_type {
            plan.modify_column_type(&new.name, old.col_type, new.col_type);
            continue;
        }

        // nullable -> not_null included: written anyway, risky if NULLs exist.
        plan.modify_column(table, old, new);
    }

    // What depends on the columns comes after them.
    for column in &change.added_columns {
        plan.create_column(table, column);
    }
    for key in &change.added_fks {
        plan.create_key(
            table,
            fk_constraint_name(table, &key.from_column, &key.to_table),
            key,
            File::Migration,
        );
    }
    for index in &change.added_indexes {
        plan.create_index(table, table, index, File::Migration);
    }

    // What destroys data closes the migration, so a failure before it
    // leaves nothing that can't be undone.
    for column in &change.dropped_columns {
        plan.delete_column(table, column);
    }

    plan
}
