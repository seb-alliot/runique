//! A table dropped.

/// `if_exists`: for the migration that drops a table, which must not fail a
/// run when the table is already gone.
pub(crate) fn table(name: &str, if_exists: bool) -> String {
    let mut down = String::new();
    down.push_str("        manager\n");
    down.push_str("            .drop_table(Table::drop()\n");
    down.push_str(&format!(
        "                .table(Alias::new(\"{}\"))\n",
        name
    ));
    if if_exists {
        down.push_str("                .if_exists()\n");
    }
    down.push_str("                .to_owned())\n");
    down.push_str("            .await?;\n");
    down
}
