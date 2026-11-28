//! An index created.
mod statement;

use crate::migration::{
    steps::generate::constraints::{fk_index_column, unless_mysql_fk},
    types::ParsedIndex,
};

/// `guard_for`: the table name the index was named for, the one it has, had
/// or will have. A foreign key's own index of that table is wrapped in the
/// MySQL check of [`unless_mysql_fk`]. `None` for a snapshot, which
/// `parser/seaorm.rs` reads back as plain statements.
pub(crate) fn index(table: &str, guard_for: Option<&str>, index: &ParsedIndex) -> String {
    let stmt = statement::statement(table, &index.name, &index.columns, index.unique);
    let mut out = match guard_for.and_then(|named_for| fk_index_column(named_for, index)) {
        Some(column) => unless_mysql_fk(table, column, &stmt),
        None => stmt,
    };
    out.push('\n');
    out
}
