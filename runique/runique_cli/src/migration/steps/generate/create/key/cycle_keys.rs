use crate::migration::steps::generate::NOT_SQLITE;
use crate::migration::steps::generate::constraints::render_fk_create;
use crate::migration::types::ParsedFk;

/// The keys that close an FK cycle, as (their table, key): added once the
/// table they point to exists, except under SQLite, which took them inline.
pub(crate) fn cycle_keys(keys: &[(&str, &ParsedFk)]) -> String {
    let mut out = format!("        if {NOT_SQLITE} {{\n");
    for (from_table, key) in keys {
        out.push_str(&format!(
            "            manager\n                .create_foreign_key(\n{}\n                        .to_owned(),\n                )\n                .await?;\n",
            render_fk_create(from_table, key, "                    ")
        ));
    }
    out.push_str("        }\n");
    out
}
