//! Destructive changes: what a run would lose, or may lose. Each case is
//! found by its own function, in the folder of its action (`create`,
//! `delete`, `modify`); here are the types and what puts the cases together.
use super::{create, delete, modify};
use crate::migration::types::Changes;
use std::fmt;

/// One thing a run does that loses data, or may: refused without `--force`.
/// A dropped table is one of them, with the dropped columns and the rest.
#[derive(Debug, Clone, PartialEq)]
pub struct Destructive {
    pub table: String,
    pub kind: DestructiveKind,
}

/// What is done to the table of a [`Destructive`].
#[derive(Debug, Clone, PartialEq)]
pub enum DestructiveKind {
    /// The table itself is dropped: every row of it is lost.
    DropTable,
    DropColumn {
        column: String,
    },
    ChangeType {
        column: String,
        from: String,
        to: String,
    },
    LeaveEnum {
        column: String,
        to: String,
    },
    EnterEnum {
        column: String,
        from: String,
    },
    MakeRequired {
        column: String,
    },
    NarrowLength {
        column: String,
        from: Option<u32>,
        to: Option<u32>,
    },
    NarrowDecimal {
        column: String,
        from: Option<(u32, u32)>,
        to: Option<(u32, u32)>,
    },
    DropKey {
        column: String,
        to_table: String,
    },
    /// A `ON DELETE CASCADE` key added to a table that already has rows.
    AddCascadeKey {
        column: String,
        to_table: String,
    },
}

/// The line shown to the developer.
impl fmt::Display for Destructive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let table = &self.table;
        match &self.kind {
            DestructiveKind::DropTable => write!(f, "  {table}: DROP TABLE (data loss)"),
            DestructiveKind::DropColumn { column } => {
                write!(f, "  {table}.{column}: DROP COLUMN (data loss)")
            }
            DestructiveKind::ChangeType { column, from, to } => {
                write!(f, "  {table}.{column}: type {from} -> {to}")
            }
            DestructiveKind::LeaveEnum { column, to } => write!(
                f,
                "  {table}.{column}: enum -> {to} (manual migration required)"
            ),
            DestructiveKind::EnterEnum { column, from } => write!(
                f,
                "  {table}.{column}: {from} -> enum (manual migration required)"
            ),
            DestructiveKind::MakeRequired { column } => write!(
                f,
                "  {table}.{column}: nullable -> not_null (requires a default or backfill)"
            ),
            DestructiveKind::NarrowLength { column, from, to } => {
                let len =
                    |l: &Option<u32>| l.map_or_else(|| "default".to_string(), |n| n.to_string());
                write!(
                    f,
                    "  {table}.{column}: length {} -> {} (longer values would be cut or refused)",
                    len(from),
                    len(to)
                )
            }
            DestructiveKind::NarrowDecimal { column, from, to } => {
                let len = |l: &Option<(u32, u32)>| {
                    l.map_or_else(|| "default".to_string(), |(p, s)| format!("({p}, {s})"))
                };
                write!(
                    f,
                    "  {table}.{column}: decimal {} -> {} (values with more digits would be rounded or refused)",
                    len(from),
                    len(to)
                )
            }
            DestructiveKind::DropKey { column, to_table } => write!(
                f,
                "  {table}.{column}: DROP FOREIGN KEY -> {to_table} (orphan records possible)"
            ),
            DestructiveKind::AddCascadeKey { column, to_table } => write!(
                f,
                "  {table}.{column}: ADD FOREIGN KEY -> {to_table} ON DELETE CASCADE (existing rows may be deleted)"
            ),
        }
    }
}

/// Everything destructive in a set of changes: the tables dropped first,
/// then, case by case, the dropped columns, the type changes, the columns
/// made required, the narrowed lengths and decimals, the dropped keys and
/// the cascade keys added to an existing table. Empty when the changes are
/// safe to apply without `--force`.
pub fn destructive(all_changes: &[Changes]) -> Vec<Destructive> {
    let mut all = delete::table(all_changes);
    all.extend(delete::column(all_changes));
    all.extend(modify::column_type(all_changes));
    all.extend(modify::column_required(all_changes));
    all.extend(modify::column_length(all_changes));
    all.extend(modify::column_decimal(all_changes));
    all.extend(delete::key(all_changes));
    all.extend(create::cascade_key(all_changes));
    all
}

/// A destructive element of the table `change` is about.
pub(crate) fn found(change: &Changes, kind: DestructiveKind) -> Destructive {
    Destructive {
        table: change.table_name.clone(),
        kind,
    }
}

/// [`destructive`] as the lines shown to the developer.
pub fn collect_destructive_messages(all_changes: &[Changes]) -> Vec<String> {
    destructive(all_changes)
        .iter()
        .map(Destructive::to_string)
        .collect()
}

/// One line per table about to be dropped, by its name alone.
pub fn dropped_table_messages(tables: &[String]) -> Vec<String> {
    tables
        .iter()
        .map(|table| {
            Destructive {
                table: table.clone(),
                kind: DestructiveKind::DropTable,
            }
            .to_string()
        })
        .collect()
}
