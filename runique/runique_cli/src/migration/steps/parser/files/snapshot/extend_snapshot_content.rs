use super::*;

/// An `extend!{}` snapshot: the extension columns, marked as recording lengths.
pub(crate) fn extend_snapshot_content(ext_schema: &ParsedSchema) -> String {
    format!(
        "{SNAPSHOT_LENGTHS_MARKER}\n{}",
        generate_create_file(ext_schema)
    )
}
