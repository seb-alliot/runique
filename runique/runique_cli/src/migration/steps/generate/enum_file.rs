//! The migration file of an enum: what creates it, or what changes its
//! values. An enum is one object, whatever the number of columns that use
//! it, and its changes are written once, here, never in the files of the
//! tables.
//!
//! The file is written for the engine of the project, which the command is
//! connected to: it holds the instructions that engine has something to run
//! for, and nothing else (`Instruction::runs_on`). PostgreSQL keeps an enum
//! as a type of its own, to create and to give values to; under MySQL and
//! SQLite the column is text, and only a renamed value has rows to write.
use super::builder::BuilderPlan;
use crate::db::DatabaseEngine;
use crate::migration::types::{EnumChange, EnumChanges};

/// Generates the migration file of an enum from what changed in it, for
/// `engine`.
///
/// `down` is `up` read backwards, each instruction replaced by its inverse:
/// a created enum is dropped, once nothing uses it; a renamed enum or value
/// takes its former name back; a value that was added stays, no engine taking a
/// value out of an enum. A value taken out is not migrated: the file says
/// so.
///
/// On an engine that has nothing to run for the enum, the file is still
/// written, and says so: applied, it is what records the enum, its name and
/// its values.
pub fn generate_enum_file(changes: &EnumChanges, engine: &DatabaseEngine) -> String {
    let name = changes.name.as_str();
    let mut plan = BuilderPlan::plan(engine);
    match changes.change {
        EnumChange::Created => {
            plan.create_enum_type(name, &changes.values);
        }
        // Without the name it had, there is nothing to rename.
        EnumChange::Renamed => {
            if let Some(from) = &changes.renamed_from {
                plan.rename_enum_type(from, name);
            }
        }
        EnumChange::Altered => {
            for (from, to) in &changes.renamed_values {
                plan.rename_enum_value(name, from, to, &changes.places);
            }
            for value in &changes.added_values {
                plan.create_enum_value(name, value);
            }
            for value in &changes.dropped_values {
                plan.delete_enum_value(name, value);
            }
        }
    }
    plan.when_nothing_runs(nothing_to_run(name)).build()
}

/// What the file holds when its engine has nothing to run for the enum.
fn nothing_to_run(name: &str) -> String {
    format!(
        "        // Nothing to run on this engine for the enum {name:?}:\n        // applied, this migration records it.\n\n"
    )
}
