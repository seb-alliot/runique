//! A column renamed. `ALTER TABLE … RENAME COLUMN` is portable (PG / MySQL 8+ / SQLite 3.25+).
pub(crate) fn column(table: &str, from: &str, to: &str) -> String {
    format!(
        "        manager\n            .alter_table(\n                Table::alter()\n                    .table(Alias::new(\"{table}\"))\n                    .rename_column(Alias::new(\"{from}\"), Alias::new(\"{to}\"))\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        from = from,
        to = to
    )
}
