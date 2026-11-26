use super::*;

/// The framework tables an `extend!{}` snapshot exists for, sorted the ones
/// the project's blocks added columns to, as last applied.
pub(crate) fn extended_tables(migrations_path: &str) -> Vec<String> {
    let Ok(files) = rust_files(Path::new(&extend_snapshot_dir(migrations_path))) else {
        return Vec::new();
    };
    files
        .into_iter()
        .filter_map(|path| path.file_stem()?.to_str().map(str::to_string))
        .collect()
}
