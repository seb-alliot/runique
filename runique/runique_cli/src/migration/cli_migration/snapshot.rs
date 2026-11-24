//! Snapshots: reading the previous state of a table, and upgrading old snapshots.
use super::scan::{merge_extend_schemas, scan_extend_blocks};
use crate::migration::helpers::file::rust_files;
use crate::migration::*;
use crate::utils::trad::tf;
use anyhow::{Context, Result};
use std::{fs, path::Path};

/// Reads a snapshot/migration file from disk and parses it into a `ParsedSchema`.
pub fn parse_create_file(path: &str) -> Result<ParsedSchema> {
    let source: String =
        fs::read_to_string(path).with_context(|| format!("Cannot read file: {}", path))?;
    parse_seaorm_source(&source).with_context(|| format!("Cannot parse: {}", path))
}

/// An `extend!{}` snapshot: the extension columns, marked as recording lengths.
pub(super) fn extend_snapshot_content(ext_schema: &ParsedSchema) -> String {
    format!(
        "{SNAPSHOT_LENGTHS_MARKER}\n{}",
        generate_create_file(ext_schema)
    )
}

fn records_lengths(path: &str) -> Result<bool> {
    Ok(has_marker(
        &fs::read_to_string(path)?,
        SNAPSHOT_LENGTHS_MARKER,
    ))
}

/// Snapshots written before column lengths were recorded, for tables this
/// run doesn't otherwise touch: rewritten once with the model's lengths, so
/// later length changes are diffed instead of adopted forever. No migration.
pub(super) fn snapshot_upgrades(
    entities_path: &str,
    schemas: &[ParsedSchema],
    main_changes: &[Changes],
    extend_planned: &[(ParsedSchema, Changes)],
    migrations_path: &str,
) -> Result<Vec<(String, String)>> {
    let mut upgrades = Vec::new();
    for schema in schemas {
        let path = snapshot_file_path(migrations_path, &schema.table_name);
        if Path::new(&path).exists()
            && !main_changes
                .iter()
                .any(|c| c.table_name == schema.table_name)
            && !records_lengths(&path)?
        {
            upgrades.push((path, generate_snapshot_file(schema)));
        }
    }
    for ext_schema in merge_extend_schemas(scan_extend_blocks(entities_path)?) {
        let path = extend_snapshot_file_path(migrations_path, &ext_schema.table_name);
        if Path::new(&path).exists()
            && !extend_planned
                .iter()
                .any(|(s, _)| s.table_name == ext_schema.table_name)
            && !records_lengths(&path)?
        {
            upgrades.push((path, extend_snapshot_content(&ext_schema)));
        }
    }
    Ok(upgrades)
}

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

/// Reads every snapshot of `snapshots/` and `snapshots/runique/`: each `.rs`
/// must parse and describe the table its file is named after. Any other file
/// is ignored. Returns the tables of `snapshots/` itself, the ones Runique
/// migrated for a model: the `extend!{}` snapshots are checked, not returned.
pub(super) fn read_all(migrations_path: &str) -> Result<Vec<ParsedSchema>> {
    let mut found = Vec::new();
    for (dir, of_a_model) in [
        (snapshot_dir(migrations_path), true),
        (extend_snapshot_dir(migrations_path), false),
    ] {
        if !Path::new(&dir).is_dir() {
            continue;
        }
        for path in rust_files(Path::new(&dir))? {
            let shown = path.display().to_string();
            let schema = parse_create_file(&shown).map_err(|_| {
                anyhow::anyhow!(tf(
                    runique_i18n::keys::makemigrations::error::SNAPSHOT_UNREADABLE,
                    &[&shown]
                ))
            })?;
            let expected = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if schema.table_name != expected {
                anyhow::bail!(
                    "{}",
                    tf(
                        runique_i18n::keys::makemigrations::error::SNAPSHOT_MISNAMED,
                        &[shown.as_str(), schema.table_name.as_str()]
                    )
                );
            }
            if of_a_model {
                found.push(schema);
            }
        }
    }
    Ok(found)
}

/// The framework tables an `extend!{}` snapshot exists for, sorted the ones
/// the project's blocks added columns to, as last applied.
pub(super) fn extended_tables(migrations_path: &str) -> Vec<String> {
    let Ok(files) = rust_files(Path::new(&extend_snapshot_dir(migrations_path))) else {
        return Vec::new();
    };
    files
        .into_iter()
        .filter_map(|path| path.file_stem()?.to_str().map(str::to_string))
        .collect()
}
