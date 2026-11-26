/// A key as a snapshot removes it: a plain statement.
pub(crate) fn snapshot_key(table: &str, name: &str) -> String {
    format!(
        "        manager\n            .drop_foreign_key(\n                ForeignKey::drop()\n                    .table(Alias::new(\"{table}\"))\n                    .name(\"{fk_name}\")\n                    .to_owned(),\n            )\n            .await?;\n\n",
        fk_name = name,
        table = table,
    )
}
