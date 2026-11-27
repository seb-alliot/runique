use super::*;

pub(crate) fn records_lengths(path: &str) -> Result<bool> {
    Ok(has_marker(
        &fs::read_to_string(path)?,
        SNAPSHOT_LENGTHS_MARKER,
    ))
}
