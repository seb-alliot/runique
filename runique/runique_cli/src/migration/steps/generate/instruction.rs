//! Every instruction a migration can hold: an action, then the object it
//! applies to. Four things are asked of an instruction, each by a `match`
//! on the action then on the object:
//!
//! - [`write`]: its text, given to the one function that writes that case;
//! - [`Instruction::order`]: when it runs in `up`;
//! - [`Instruction::inverse`]: what undoes it;
//! - [`Instruction::runs_on`]: whether an engine has anything to run for it;
//! - [`body`]: a whole `up` or `down`, by its [`Direction`], for an engine.
//!
//! A `down` is never written on its own: it is the `up` read backwards, each
//! instruction replaced by its inverse.
use super::{constraints::fk_constraint_name, create, delete, modify, rename};
use crate::{
    db::DatabaseEngine,
    migration::types::{ColumnType, EnumPlace, ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema},
};

/// Which body of a migration is written.
#[derive(Clone, Copy)]
pub(super) enum Direction {
    Up,
    Down,
}

/// Which file an instruction is written for: a migration, which runs, or a
/// snapshot, which `parser/seaorm.rs` reads back and so keeps to plain
/// statements (keys apart from their table, no check around an index).
#[derive(Clone, Copy)]
pub(super) enum File {
    Migration,
    Snapshot,
}

/// When an instruction runs in `up`, from first to last. What is in the way
/// goes before the columns change, what depends on them comes after, and
/// what destroys data closes the migration. `down` runs the other way.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Order {
    CreateEnumTypes,
    RenameEnumTypes,
    CreateTable,
    RenameTable,
    RenameIndexes,
    RenameKeys,
    RenameColumns,
    DeleteIndexes,
    DeleteKeys,
    ModifyColumns,
    CreateColumns,
    CreateKeys,
    CreateIndexes,
    CreateCycleKeys,
    RenameEnumValues,
    CreateEnumValues,
    DeleteEnumValues,
    DeleteColumns,
    DeleteTable,
    DeleteEnumTypes,
}

pub(super) enum Instruction<'a> {
    Create(Create<'a>),
    Delete(Delete<'a>),
    Modify(Modify<'a>),
    Rename(Rename<'a>),
}

pub(super) enum Create<'a> {
    Table {
        schema: &'a ParsedSchema,
        forward: &'a [&'a ParsedFk],
        file: File,
    },
    Column {
        table: &'a str,
        column: &'a ParsedColumn,
    },
    Key {
        table: &'a str,
        name: String,
        key: &'a ParsedFk,
        file: File,
    },
    CycleKeys {
        keys: &'a [(&'a str, &'a ParsedFk)],
    },
    /// `named_for`: the table name the index is named for, the one it has,
    /// had or will have.
    Index {
        table: &'a str,
        named_for: &'a str,
        index: &'a ParsedIndex,
        file: File,
    },
    EnumType {
        name: &'a str,
        values: &'a [String],
    },
    EnumValue {
        enum_name: &'a str,
        value: &'a str,
    },
}

/// A deletion holds the whole object, not only its name: turned round, it
/// is what creates the object again.
pub(super) enum Delete<'a> {
    Table {
        name: &'a str,
        if_exists: bool,
    },
    Column {
        table: &'a str,
        column: &'a ParsedColumn,
    },
    Key {
        table: &'a str,
        name: String,
        key: &'a ParsedFk,
        file: File,
    },
    CycleKeys {
        keys: &'a [(&'a str, &'a ParsedFk)],
    },
    /// `named_for`: the table name the index is named for, the one it has,
    /// had or will have.
    Index {
        table: &'a str,
        named_for: &'a str,
        index: &'a ParsedIndex,
        file: File,
    },
    EnumType {
        name: &'a str,
        values: &'a [String],
    },
    EnumValue {
        enum_name: &'a str,
        value: &'a str,
    },
}

pub(super) enum Modify<'a> {
    Column {
        table: &'a str,
        from: &'a ParsedColumn,
        to: &'a ParsedColumn,
    },
    ColumnType {
        column: &'a str,
        from: ColumnType,
        to: ColumnType,
    },
    EnumColumn {
        table: &'a str,
        from: &'a ParsedColumn,
        to: &'a ParsedColumn,
    },
}

pub(super) enum Rename<'a> {
    Table {
        from: &'a str,
        to: &'a str,
    },
    Column {
        table: &'a str,
        from: &'a str,
        to: &'a str,
    },
    /// `from` and `to`: the (table, target) names the key is named for,
    /// before and after. `target`: the name its target has at that moment.
    Key {
        table: &'a str,
        from: (&'a str, &'a str),
        to: (&'a str, &'a str),
        key: &'a ParsedFk,
        target: &'a str,
    },
    /// `from` and `to`: the index, and the table name it is named for.
    Index {
        table: &'a str,
        from: (&'a ParsedIndex, &'a str),
        to: (&'a ParsedIndex, &'a str),
    },
    EnumType {
        from: &'a str,
        to: &'a str,
    },
    /// Once for the enum: `places` are every column that uses it.
    EnumValue {
        enum_name: &'a str,
        from: &'a str,
        to: &'a str,
        places: &'a [EnumPlace],
    },
}

/// The table name the MySQL check of a key's own index looks for: none in a
/// snapshot, which never carries that check.
fn guard(named_for: &str, file: File) -> Option<&str> {
    match file {
        File::Migration => Some(named_for),
        File::Snapshot => None,
    }
}

/// The text of one instruction, as it stands in a migration file.
pub(super) fn write(instruction: &Instruction<'_>) -> String {
    match instruction {
        Instruction::Create(what) => match what {
            Create::Table {
                schema,
                forward,
                file,
            } => create::table(schema, forward, *file),
            Create::Column { table, column } => create::column(table, column),
            Create::Key {
                table,
                name,
                key,
                file,
            } => match file {
                File::Migration => create::key(table, name, key),
                File::Snapshot => create::snapshot_key(table, name, key),
            },
            Create::CycleKeys { keys } => create::cycle_keys(keys),
            Create::Index {
                table,
                named_for,
                index,
                file,
            } => create::index(table, guard(named_for, *file), index),
            Create::EnumType { name, values } => create::enum_type(name, values),
            Create::EnumValue { enum_name, value } => create::enum_value(enum_name, value),
        },
        Instruction::Delete(what) => match what {
            Delete::Table { name, if_exists } => delete::table(name, *if_exists),
            Delete::Column { table, column } => delete::column(table, &column.name),
            Delete::Key {
                table, name, file, ..
            } => match file {
                File::Migration => delete::key(table, name),
                File::Snapshot => delete::snapshot_key(table, name),
            },
            Delete::CycleKeys { keys } => delete::cycle_keys(keys),
            Delete::Index {
                table,
                named_for,
                index,
                file,
            } => delete::index(table, guard(named_for, *file), index),
            Delete::EnumType { name, .. } => delete::enum_type(name),
            Delete::EnumValue { enum_name, value } => delete::enum_value(enum_name, value),
        },
        Instruction::Modify(what) => match what {
            Modify::Column { table, to, .. } => modify::column(table, to),
            Modify::ColumnType { column, from, to } => modify::column_type(column, *from, *to),
            Modify::EnumColumn { table, from, to } => modify::enum_column(table, from, to),
        },
        Instruction::Rename(what) => match what {
            Rename::Table { from, to } => rename::table(from, to),
            Rename::Column { table, from, to } => rename::column(table, from, to),
            Rename::Key {
                table,
                from,
                to,
                key,
                target,
            } => rename::key(
                table,
                &fk_constraint_name(from.0, &key.from_column, from.1),
                *to,
                key,
                target,
            ),
            Rename::Index { table, from, to } => rename::index(table, *from, *to),
            Rename::EnumType { from, to } => rename::enum_type(from, to),
            Rename::EnumValue {
                enum_name,
                from,
                to,
                places,
            } => rename::enum_value(enum_name, from, to, places),
        },
    }
}

impl<'a> Instruction<'a> {
    /// When this instruction runs in `up`: its action and its object say it.
    pub(super) fn order(&self) -> Order {
        match self {
            Instruction::Create(what) => match what {
                Create::EnumType { .. } => Order::CreateEnumTypes,
                Create::Table { .. } => Order::CreateTable,
                Create::Column { .. } => Order::CreateColumns,
                Create::Key { .. } => Order::CreateKeys,
                Create::Index { .. } => Order::CreateIndexes,
                Create::CycleKeys { .. } => Order::CreateCycleKeys,
                Create::EnumValue { .. } => Order::CreateEnumValues,
            },
            Instruction::Delete(what) => match what {
                Delete::Index { .. } => Order::DeleteIndexes,
                Delete::Key { .. } | Delete::CycleKeys { .. } => Order::DeleteKeys,
                Delete::EnumValue { .. } => Order::DeleteEnumValues,
                Delete::Column { .. } => Order::DeleteColumns,
                Delete::Table { .. } => Order::DeleteTable,
                Delete::EnumType { .. } => Order::DeleteEnumTypes,
            },
            Instruction::Modify(what) => match what {
                Modify::Column { .. } | Modify::ColumnType { .. } | Modify::EnumColumn { .. } => {
                    Order::ModifyColumns
                }
            },
            Instruction::Rename(what) => match what {
                Rename::Table { .. } => Order::RenameTable,
                Rename::Index { .. } => Order::RenameIndexes,
                Rename::Key { .. } => Order::RenameKeys,
                Rename::Column { .. } => Order::RenameColumns,
                Rename::EnumType { .. } => Order::RenameEnumTypes,
                Rename::EnumValue { .. } => Order::RenameEnumValues,
            },
        }
    }

    /// What undoes this instruction: what was created is deleted, what was
    /// deleted is created again, and a change or a rename is the same case
    /// with its two values the other way round. `None` when nothing undoes
    /// it.
    pub(super) fn inverse(&self) -> Option<Instruction<'a>> {
        Some(match self {
            Instruction::Create(what) => Instruction::Delete(match *what {
                Create::Table { schema, .. } => Delete::Table {
                    name: &schema.table_name,
                    if_exists: false,
                },
                Create::Column { table, column } => Delete::Column { table, column },
                Create::Key {
                    table,
                    ref name,
                    key,
                    file,
                } => Delete::Key {
                    table,
                    name: name.clone(),
                    key,
                    file,
                },
                Create::CycleKeys { keys } => Delete::CycleKeys { keys },
                Create::Index {
                    table,
                    named_for,
                    index,
                    file,
                } => Delete::Index {
                    table,
                    named_for,
                    index,
                    file,
                },
                Create::EnumType { name, values } => Delete::EnumType { name, values },
                // Postgres cannot take a value out of an enum type.
                Create::EnumValue { .. } => return None,
            }),
            Instruction::Delete(what) => Instruction::Create(match *what {
                // The file that drops a table writes its way back itself,
                // from the table's snapshot.
                Delete::Table { .. } => return None,
                Delete::Column { table, column } => Create::Column { table, column },
                Delete::Key {
                    table,
                    ref name,
                    key,
                    file,
                } => Create::Key {
                    table,
                    name: name.clone(),
                    key,
                    file,
                },
                Delete::CycleKeys { keys } => Create::CycleKeys { keys },
                Delete::Index {
                    table,
                    named_for,
                    index,
                    file,
                } => Create::Index {
                    table,
                    named_for,
                    index,
                    file,
                },
                Delete::EnumType { name, values } => Create::EnumType { name, values },
                Delete::EnumValue { enum_name, value } => Create::EnumValue { enum_name, value },
            }),
            Instruction::Modify(what) => Instruction::Modify(match *what {
                Modify::Column { table, from, to } => Modify::Column {
                    table,
                    from: to,
                    to: from,
                },
                // Not migrated yet: nothing was done, nothing is undone.
                Modify::ColumnType { .. } => return None,
                Modify::EnumColumn { table, from, to } => Modify::EnumColumn {
                    table,
                    from: to,
                    to: from,
                },
            }),
            Instruction::Rename(what) => Instruction::Rename(match *what {
                Rename::Table { from, to } => Rename::Table { from: to, to: from },
                Rename::Column { table, from, to } => Rename::Column {
                    table,
                    from: to,
                    to: from,
                },
                Rename::Key {
                    table,
                    from,
                    to,
                    key,
                    target,
                } => Rename::Key {
                    table,
                    from: to,
                    to: from,
                    key,
                    target,
                },
                Rename::Index { table, from, to } => Rename::Index {
                    table,
                    from: to,
                    to: from,
                },
                Rename::EnumType { from, to } => Rename::EnumType { from: to, to: from },
                Rename::EnumValue {
                    enum_name,
                    from,
                    to,
                    places,
                } => Rename::EnumValue {
                    enum_name,
                    from: to,
                    to: from,
                    places,
                },
            }),
        })
    }

    /// Whether `engine` has anything to run for this instruction. Only
    /// PostgreSQL keeps an enum as an object of its own, with a type to
    /// create, to rename, to give a value to and to drop: under MySQL and
    /// SQLite an enum column is text, and of an enum only a renamed value
    /// has something to write, in the rows. Everything else runs everywhere.
    pub(super) fn runs_on(&self, engine: &DatabaseEngine) -> bool {
        match engine {
            DatabaseEngine::PostgreSQL => true,
            DatabaseEngine::MySQL | DatabaseEngine::SQLite => !matches!(
                self,
                Instruction::Create(Create::EnumType { .. } | Create::EnumValue { .. })
                    | Instruction::Delete(Delete::EnumType { .. })
                    | Instruction::Rename(Rename::EnumType { .. })
            ),
        }
    }
}

/// The instructions of a migration body, one after the other, as given.
pub(super) fn chain(instructions: &[Instruction<'_>]) -> String {
    instructions.iter().map(write).collect()
}

/// A body of a migration, from the instructions of its `up` in any order:
/// `Up` puts them in theirs; `Down` runs that order backwards, each
/// instruction replaced by its inverse. With an `engine`, the body holds
/// what that engine has something to run for and nothing else; without one
/// it holds every instruction, for a file that is not written for an engine.
pub(super) fn body(
    instructions: &[Instruction<'_>],
    direction: Direction,
    engine: Option<&DatabaseEngine>,
) -> String {
    let wanted = |instruction: &Instruction<'_>| engine.is_none_or(|e| instruction.runs_on(e));
    let mut ordered: Vec<&Instruction<'_>> = instructions.iter().collect();
    // Stable: the instructions of one step keep the order they were given in.
    ordered.sort_by_key(|instruction| instruction.order());
    match direction {
        Direction::Up => ordered
            .into_iter()
            .filter(|instruction| wanted(instruction))
            .map(write)
            .collect(),
        Direction::Down => ordered
            .into_iter()
            .rev()
            .filter_map(|instruction| instruction.inverse())
            .filter(wanted)
            .map(|instruction| write(&instruction))
            .collect(),
    }
}
