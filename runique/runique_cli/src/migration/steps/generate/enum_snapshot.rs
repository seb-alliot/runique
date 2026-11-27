//! The snapshot of an enum: the state of its values once `migration up`
//! applied them. Written as the creation of its type, the way a table's
//! snapshot is written as its creation: read back, never compiled, and the
//! same on every engine.
use super::migration_file;
use crate::migration::types::ParsedEnum;

/// Generates the snapshot file of an enum.
pub fn generate_enum_snapshot_file(parsed: &ParsedEnum) -> String {
    let name = &parsed.name;
    let values = parsed
        .values
        .iter()
        .map(|value| format!("Alias::new({value:?})"))
        .collect::<Vec<_>>()
        .join(", ");
    let up = format!(
        "        manager\n            .create_type(\n                sea_query::extension::postgres::Type::create()\n                    .as_enum(Alias::new({name:?}))\n                    .values([{values}])\n                    .to_owned(),\n            )\n            .await?;\n        Ok(())\n"
    );
    let down = format!(
        "        manager\n            .drop_type(\n                sea_query::extension::postgres::Type::drop()\n                    .name(Alias::new({name:?}))\n                    .to_owned(),\n            )\n            .await?;\n        Ok(())\n"
    );
    migration_file(&up, &down)
}
