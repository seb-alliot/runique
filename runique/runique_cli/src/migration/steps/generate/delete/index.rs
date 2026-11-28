//! An index dropped.
use crate::migration::{
    steps::generate::constraints::{fk_index_column, unless_mysql_fk},
    types::ParsedIndex,
};

/// `guard_for`: as for the creation of an index (`create::index`).
pub(crate) fn index(table: &str, guard_for: Option<&str>, index: &ParsedIndex) -> String {
    let stmt = format!(
        "        manager\n            .drop_index(Index::drop().name(\"{name}\").table(Alias::new(\"{table}\")).to_owned())\n            .await?;\n",
        name = index.name,
        table = table
    );
    let mut out = match guard_for.and_then(|named_for| fk_index_column(named_for, index)) {
        Some(column) => unless_mysql_fk(table, column, &stmt),
        None => stmt,
    };
    out.push('\n');
    out
}
