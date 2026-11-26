//! A table renamed.

/// `Table::rename()` from `from` to `to`.
pub(crate) fn table(from: &str, to: &str) -> String {
    format!(
        "        manager\n            .rename_table(\n                Table::rename()\n                    .table(Alias::new(\"{from}\"), Alias::new(\"{to}\"))\n                    .to_owned(),\n            )\n            .await?;\n\n"
    )
}
