//! A column removed: the one instruction of an ALTER that loses data.
pub(crate) fn column(table: &str, name: &str) -> String {
    format!(
        "        manager\n            .alter_table(\n                Table::alter()\n                    .table(Alias::new(\"{table}\"))\n                    .drop_column(Alias::new(\"{col}\"))\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        col = name
    )
}
