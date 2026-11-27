use super::*;

/// The enums the database has, as the snapshots tell: those a snapshot of a
/// table still uses, and no other. `tables` are the snapshots of the tables
/// of the models and of the extensions, `enum_snapshots` those of the enums.
///
/// An enum has the values of its own snapshot. Without one, it has those its
/// column was written with: a project migrated before enums had snapshots.
/// A snapshot of an enum no table uses is left out: its type went with the
/// last column that used it, and declared again it is created again.
pub fn previous_enums(tables: &[ParsedSchema], enum_snapshots: &[ParsedEnum]) -> Vec<ParsedEnum> {
    enums_of_schemas(tables)
        .into_iter()
        .map(|used| {
            match enum_snapshots
                .iter()
                .find(|snapshot| snapshot.name == used.name)
            {
                Some(snapshot) => ParsedEnum {
                    values: snapshot.values.clone(),
                    ..used
                },
                None => used,
            }
        })
        .collect()
}

/// Gives the enum columns of `schema`, a snapshot of a table, the values
/// their enum has in `enums`: the snapshot of a table keeps the values its
/// columns were written with, which a later migration of the enum doesn't
/// rewrite there.
pub fn with_enum_values(schema: &mut ParsedSchema, enums: &[ParsedEnum]) {
    for column in &mut schema.columns {
        if column.enum_string_values.is_empty() {
            continue;
        }
        let name = column
            .enum_name
            .as_deref()
            .unwrap_or(&column.name)
            .to_lowercase();
        if let Some(known) = enums.iter().find(|known| known.name == name) {
            column.enum_string_values = known.values.clone();
        }
    }
}

/// The snapshots to write for the enums of `previous` that have none yet,
/// as (path, content): a project migrated before enums had snapshots gets
/// them once, with the values its tables were written with. No migration.
pub fn enum_snapshot_upgrades(
    previous: &[ParsedEnum],
    enum_snapshots: &[ParsedEnum],
    migrations_path: &str,
) -> Vec<(String, String)> {
    previous
        .iter()
        .filter(|known| {
            !enum_snapshots
                .iter()
                .any(|snapshot| snapshot.name == known.name)
        })
        .map(|known| {
            (
                enum_snapshot_file_path(migrations_path, &known.name),
                generate_enum_snapshot_file(known),
            )
        })
        .collect()
}
