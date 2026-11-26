use super::*;

/// Writes the whole plan as one batch — generated files, replaced migrations
/// removed, and the `lib.rs` that registers them: all of it, or the project
/// as it was.
pub(crate) fn commit_plan(plan: &Plan, migrations_path: &str) -> Result<()> {
    let mut batch = AtomicBuilder::modify().journal(migrations_path);
    for (path, content) in &plan.files {
        batch = batch.add(
            PathBuf::from(path),
            SourceItems::Bytes(content.clone().into_bytes()),
            false,
        )?;
    }
    for module in &plan.dropped_modules {
        batch = batch.add(
            PathBuf::from(format!("{migrations_path}/{module}.rs")),
            SourceItems::Remove,
            false,
        )?;
    }
    if let Some(lib) = planned_lib(migrations_path, &plan.lib_modules, &plan.dropped_modules)? {
        batch = batch.add(
            PathBuf::from(lib_path(migrations_path)),
            SourceItems::Bytes(lib.into_bytes()),
            false,
        )?;
    }
    batch.start()?;
    // Runs whether `lib.rs` pre-existed or not: `sea-orm-cli migrate init` always
    // leaves this placeholder behind, and the `lib.rs` just written no longer names it.
    strip_sea_orm_cli_placeholder_file(migrations_path);
    Ok(())
}
