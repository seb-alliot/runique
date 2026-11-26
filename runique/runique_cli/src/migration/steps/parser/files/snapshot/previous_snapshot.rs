use super::*;

/// The previous snapshot of a table, to diff `current` against. A snapshot
/// written before column lengths were recorded doesn't know them: it takes
/// `current`'s, so upgrading never produces a length migration on its own.
pub(crate) fn previous_snapshot(path: &str, current: &ParsedSchema) -> Result<ParsedSchema> {
    let mut previous = parse_create_file(path)?;
    if !records_lengths(path)? {
        for col in &mut previous.columns {
            if let Some(cur) = current.columns.iter().find(|c| c.name == col.name) {
                col.max_length = cur.max_length;
            }
        }
    }
    Ok(previous)
}
