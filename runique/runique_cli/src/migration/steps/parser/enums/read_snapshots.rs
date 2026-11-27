use super::*;

/// Reads every snapshot of `snapshots/enums/`: each `.rs` must parse and
/// describe the enum its file is named after. Any other file is ignored, and
/// a project without the folder has no enum snapshot yet.
pub fn read_enum_snapshots(migrations_path: &str) -> Result<Vec<ParsedEnum>> {
    let dir = enum_snapshot_dir(migrations_path);
    if !Path::new(&dir).is_dir() {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    for path in rust_files(Path::new(&dir))? {
        let unreadable = || {
            anyhow::anyhow!(tf(
                runique_i18n::keys::makemigrations::error::SNAPSHOT_UNREADABLE,
                &[path.display()]
            ))
        };
        let source = fs::read_to_string(&path).map_err(|_| unreadable())?;
        let parsed = parse_enum_snapshot(&source).map_err(|_| unreadable())?;
        if path.file_stem().and_then(|stem| stem.to_str()) != Some(parsed.name.as_str()) {
            anyhow::bail!(
                "{}",
                tf(
                    runique_i18n::keys::makemigrations::error::ENUM_SNAPSHOT_MISNAMED,
                    &[path.display().to_string().as_str(), parsed.name.as_str()]
                )
            );
        }
        found.push(parsed);
    }
    Ok(found)
}
