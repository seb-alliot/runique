//! A column added to an existing table.
use crate::migration::steps::generate::columns::render_column_def;
use crate::migration::types::ParsedColumn;

pub(crate) fn column(table: &str, column: &ParsedColumn) -> String {
    format!(
        "        manager\n            .alter_table(\n                Table::alter()\n                    .table(Alias::new(\"{table}\"))\n                    .add_column({coldef})\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        coldef = render_column_def(column),
    )
}
