use crate::migration::{
    steps::generate::{NOT_SQLITE, constraints::fk_constraint_name},
    types::ParsedFk,
};

/// The keys that close an FK cycle, as (their table, key): removed before
/// the table they point to, except under SQLite, where they go with theirs.
pub(crate) fn cycle_keys(keys: &[(&str, &ParsedFk)]) -> String {
    let mut out = format!("        if {NOT_SQLITE} {{\n");
    for (from_table, key) in keys {
        out.push_str(&format!(
            "            manager\n                .drop_foreign_key(\n                    ForeignKey::drop()\n                        .table(Alias::new(\"{from_table}\"))\n                        .name(\"{name}\")\n                        .to_owned(),\n                )\n                .await?;\n",
            name = fk_constraint_name(from_table, &key.from_column, &key.to_table),
        ));
    }
    out.push_str("        }\n");
    out
}
