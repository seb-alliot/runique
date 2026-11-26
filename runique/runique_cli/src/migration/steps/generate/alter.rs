//! The `up`/`down` bodies of an ALTER migration: the instructions a change
//! of table asks for. Their order and their way back are the instructions' own.
use super::constraints::fk_constraint_name;
use super::enum_types;
use super::instruction::{Create, Delete, Direction, File, Instruction, Modify, Rename, body};
use crate::migration::types::Changes;

/// True if `body` has a line that isn't blank or a `//` comment — i.e. it actually calls `manager`.
/// A type-change diff only emits a `// Manual migration required.` comment, which is non-empty
/// text but no real statement, so a plain `is_empty()` check picks `manager` and leaves it unused.
pub(super) fn body_has_code(body: &str) -> bool {
    body.lines()
        .any(|line| !line.trim().is_empty() && !line.trim_start().starts_with("//"))
}

pub(super) fn build_alter_bodies(change: &Changes) -> (String, String) {
    // The instructions of `up`, as they come: their order is theirs to tell
    // (`Instruction::order`), and `down` is `up` read backwards.
    let mut up: Vec<Instruction> = Vec::new();
    let table = change.table_name.as_str();
    // The name the table had when its keys and indexes were named.
    let former = change.renames.from.as_deref().unwrap_or(table);
    let enums = enum_types(&change.added_columns);

    // The enum types of the added columns exist before anything uses them.
    for (name, values) in &enums {
        up.push(Instruction::Create(Create::EnumType { name, values }));
    }

    // The table takes its new name, then the indexes and keys named after
    // the former one take theirs.
    if let Some(old) = &change.renames.from {
        up.push(Instruction::Rename(Rename::Table {
            from: old,
            to: table,
        }));
    }
    for (old, new) in &change.renames.indexes {
        up.push(Instruction::Rename(Rename::Index {
            table,
            from: (old, former),
            to: (new, table),
        }));
    }
    for key in &change.renames.keys {
        up.push(Instruction::Rename(Rename::Key {
            table,
            from: (key.old_table.as_str(), key.old_target.as_str()),
            to: (table, key.key.to_table.as_str()),
            key: &key.key,
            target: &key.target_at_run,
        }));
    }
    for (old, new) in &change.renamed_columns {
        up.push(Instruction::Rename(Rename::Column {
            table,
            from: old,
            to: new,
        }));
    }

    // What is in the way of the columns goes first. A key of a renamed
    // table that goes still has the name it was given under the former
    // table name, and so does its index.
    for index in &change.dropped_indexes {
        up.push(Instruction::Delete(Delete::Index {
            table,
            named_for: former,
            index,
            file: File::Migration,
        }));
    }
    for key in &change.dropped_fks {
        up.push(Instruction::Delete(Delete::Key {
            table,
            name: fk_constraint_name(former, &key.from_column, &key.to_table),
            key,
            file: File::Migration,
        }));
    }

    for (old, new) in &change.modified_columns {
        // Becoming (or stopping being) an enum: `col_type` alone doesn't capture this
        // (an enum column keeps col_type "String"), so this must be checked before the
        // generic "type change => manual" fallback below, which would otherwise catch
        // ordinary type changes but silently miss this one entirely.
        if old.enum_string_values.is_empty() != new.enum_string_values.is_empty() {
            up.push(Instruction::Modify(Modify::EnumColumn {
                table,
                from: old,
                to: new,
            }));
            continue;
        }

        // type change => manual
        if old.col_type != new.col_type {
            up.push(Instruction::Modify(Modify::ColumnType {
                column: &new.name,
                from: old.col_type,
                to: new.col_type,
            }));
            continue;
        }

        // nullable -> not_null included: written anyway, risky if NULLs exist.
        up.push(Instruction::Modify(Modify::Column {
            table,
            from: old,
            to: new,
        }));
    }

    // What depends on the columns comes after them.
    for column in &change.added_columns {
        up.push(Instruction::Create(Create::Column { table, column }));
    }
    for key in &change.added_fks {
        up.push(Instruction::Create(Create::Key {
            table,
            name: fk_constraint_name(table, &key.from_column, &key.to_table),
            key,
            file: File::Migration,
        }));
    }
    for index in &change.added_indexes {
        up.push(Instruction::Create(Create::Index {
            table,
            named_for: table,
            index,
            file: File::Migration,
        }));
    }

    for (column, enum_name, old, new) in &change.enum_renames {
        up.push(Instruction::Rename(Rename::EnumValue {
            table,
            column,
            enum_name,
            from: old,
            to: new,
        }));
    }
    for (_column, enum_name, value) in &change.enum_value_adds {
        up.push(Instruction::Create(Create::EnumValue { enum_name, value }));
    }
    for (_column, enum_name, value) in &change.enum_value_drops {
        up.push(Instruction::Delete(Delete::EnumValue { enum_name, value }));
    }

    // What destroys data closes the migration, so a failure before it
    // leaves nothing that can't be undone.
    for column in &change.dropped_columns {
        up.push(Instruction::Delete(Delete::Column { table, column }));
    }

    (body(&up, Direction::Up), body(&up, Direction::Down))
}
