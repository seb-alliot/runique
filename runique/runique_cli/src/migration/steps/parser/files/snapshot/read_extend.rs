use super::*;

/// The snapshots of `snapshots/runique/`: what the project's `extend!{}`
/// blocks added to the framework's tables, as last applied.
pub(crate) fn read_extend(migrations_path: &str) -> Result<Vec<ParsedSchema>> {
    extended_tables(migrations_path)
        .iter()
        .map(|table| parse_create_file(&extend_snapshot_file_path(migrations_path, table)))
        .collect()
}
