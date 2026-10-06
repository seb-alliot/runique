//! `makemigration` command — generates SeaORM migration files from `ModelSchema` DSL.
use crate::migration::*;
use crate::utils::trad::{t, tf};
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs;
use std::path::Path;

pub use crate::utils::*;

// ── public parse entry points ────────────────────────────────────────────────

/// Reads a snapshot/migration file from disk and parses it into a `ParsedSchema`.
pub fn parse_create_file(path: &str) -> Result<ParsedSchema> {
    let source: String =
        fs::read_to_string(path).with_context(|| format!("Cannot read file: {}", path))?;
    parse_seaorm_source(&source).with_context(|| format!("Cannot parse: {}", path))
}

/// An `extend!{}` snapshot: the extension columns, marked as recording lengths.
fn extend_snapshot_content(ext_schema: &ParsedSchema) -> String {
    format!(
        "{SNAPSHOT_LENGTHS_MARKER}\n{}",
        generate_create_file(ext_schema)
    )
}

fn records_lengths(path: &str) -> Result<bool> {
    Ok(fs::read_to_string(path)?.starts_with(SNAPSHOT_LENGTHS_MARKER))
}

/// Snapshots written before column lengths were recorded, for tables this
/// run doesn't otherwise touch: rewritten once with the model's lengths, so
/// later length changes are diffed instead of adopted forever. No migration.
fn snapshot_upgrades(
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

// ── scan ─────────────────────────────────────────────────────────────────────

/// Tables created by `EihwazUsersMigration` + `AdminTableMigration`, never
/// migrated from the app's entities.
const FRAMEWORK_TABLES: &[&str] = &[
    "eihwaz_users",
    "eihwaz_groupes",
    "eihwaz_groupes_droits",
    "eihwaz_users_groupes",
    "eihwaz_sessions",
    "eihwaz_reset_tokens",
];

/// Scans every `.rs` file in `entities_path` for `model!{}` schemas and
/// parses each one. A `belongs_to` target is resolved the way the `model!{}`
/// macro resolves it — the entity module of that name, i.e. the file
/// `<target>.rs`, or a framework table — and its FK then references that
/// model's real table and primary key; a target that matches nothing is an
/// error. Skips the framework's own tables (`eihwaz_*`).
pub fn scan_entities(entities_path: &str) -> Result<Vec<ParsedSchema>> {
    let mut schemas = Vec::new();
    // List fields' tables: their key to the owner is already resolved.
    let mut list_schemas: Vec<ParsedSchema> = Vec::new();
    // module name → (table, primary key column)
    let mut targets: std::collections::HashMap<String, (String, String)> = FRAMEWORK_TABLES
        .iter()
        .map(|t| (t.to_string(), (t.to_string(), "id".to_string())))
        .collect();
    let mut entries: Vec<_> = fs::read_dir(entities_path)
        .with_context(|| format!("Cannot read entities directory: {}", entities_path))?
        .collect::<std::io::Result<_>>()?;
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let Some(module) = path.file_stem().and_then(|n| n.to_str()) else {
            continue;
        };
        if module == "mod" {
            continue;
        }

        let source = fs::read_to_string(&path)
            .with_context(|| format!("Cannot read file: {}", path.display()))?;

        if let Some(ParsedModel { schema, lists, .. }) = parse_model_from_source(&source)
            .with_context(|| format!("Invalid model!{{}} in {}", path.display()))?
        {
            let pk = schema
                .primary_key
                .as_ref()
                .map_or_else(|| "id".to_string(), |pk| pk.name.clone());
            targets.insert(module.to_string(), (schema.table_name.clone(), pk));
            // Ignore tables provided by the framework (`EihwazUsersMigration` + `AdminTableMigration`)
            if FRAMEWORK_TABLES.contains(&schema.table_name.as_str()) {
                continue;
            }
            schemas.push(schema);
            list_schemas.extend(lists);
        }
    }

    for schema in &mut schemas {
        for fk in &mut schema.foreign_keys {
            let Some((table, pk)) = targets.get(&fk.to_table) else {
                anyhow::bail!(
                    "{}: `belongs_to: {} via {}` — no model `{}` ({}/{}.rs) and no framework table of that name",
                    schema.table_name,
                    fk.to_table,
                    fk.from_column,
                    fk.to_table,
                    entities_path,
                    fk.to_table
                );
            };
            fk.to_table = table.clone();
            fk.to_column = pk.clone();
        }
    }

    schemas.extend(list_schemas);
    let mut seen = std::collections::HashSet::new();
    for schema in &schemas {
        if !seen.insert(schema.table_name.as_str()) {
            anyhow::bail!(
                "two tables are named `{}` — a list field's table is `<owner table>_<field>`: rename the field or the other table",
                schema.table_name
            );
        }
    }

    Ok(schemas)
}

// ── lib.rs updater ───────────────────────────────────────────────────────────

/// `module_name` must be in SeaORM format: `m{timestamp}_create_{table}_table`
pub fn update_migration_lib(migrations_path: &str, module_name: &str) -> Result<()> {
    let lib = lib_path(migrations_path);
    let box_entry = format!("Box::new({}::Migration)", module_name);

    let mut state = if Path::new(&lib).exists() {
        parse_lib_state(&fs::read_to_string(&lib)?)
    } else {
        LibState::default()
    };

    // Runs whether `lib.rs` pre-existed or not: `sea-orm-cli migrate init` always
    // leaves this exact placeholder wired in (its `up()`/`down()` are both a bare
    // `todo!()`, which panics the first time any migration actually runs).
    strip_sea_orm_cli_placeholder_file(migrations_path);

    if state.mods.contains(&module_name.to_string()) {
        return Ok(());
    }
    state.mods.push(module_name.to_string());
    state
        .entries
        .retain(|e| !e.contains(SEA_ORM_CLI_PLACEHOLDER_MODULE));
    state.entries.push(box_entry);

    fs::write(&lib, render_lib(&state))?;
    Ok(())
}

// ── lib.rs canonical rewriting ───────────────────────────────────────────────
//
// `sea-orm-cli migrate init` is the documented first step to bootstrap the
// `migration` crate (Cargo.toml + main.rs) before ever running `makemigrations`
// — its own generated `lib.rs` uses a different shape than Runique's (single-line
// `vec![Box::new(X::Migration)]`, `pub use sea_orm_migration::prelude::*;`) that
// broke the old line-based `.replacen(...)` patching: the `vec![` search matched
// but inserted new entries *after* the whole (already-closed) vec statement
// instead of inside it, and framework migrations landed outside `Migrator::migrations()`
// entirely — syntactically invalid output. Parsing into a small in-memory model
// and always re-rendering canonically sidesteps every formatting assumption.

/// Literal placeholder module name `sea-orm-cli migrate init` always generates
/// (a fixed string, not templated with a timestamp — confirmed by direct testing).
const SEA_ORM_CLI_PLACEHOLDER_MODULE: &str = "m20220101_000001_create_table";

/// Parsed, formatting-independent view of `Migrator::migrations()`'s `lib.rs`.
#[derive(Default)]
struct LibState {
    uses_migrations_table: bool,
    mods: Vec<String>,
    entries: Vec<String>,
}

/// Finds the byte range of the `vec![ ... ]` inside `Migrator::migrations()`,
/// by bracket-depth balance from the first `vec![` — robust whether it's
/// Runique's own multi-line output or a single-line `vec![Box::new(...)]`.
fn find_vec_span(content: &str) -> Option<(usize, usize)> {
    let start = content.find("vec![")?;
    let open = start + "vec![".len() - 1;
    let mut depth = 0i32;
    for (i, ch) in content[open..].char_indices() {
        match ch {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some((open, open + i + 1));
                }
            }
            _ => {}
        }
    }
    None
}

/// Extracts each `Box::new(...)` entry inside a `vec![...]` span, in order —
/// works regardless of the original line layout (one entry per line, several
/// per line, or all on a single line).
fn extract_box_entries(vec_content: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut rest = vec_content;
    while let Some(start) = rest.find("Box::new(") {
        let after = &rest[start + "Box::new(".len()..];
        let Some(end) = after.find(')') else { break };
        entries.push(format!("Box::new({})", &after[..end]));
        rest = &after[end + 1..];
    }
    entries
}

fn parse_lib_state(content: &str) -> LibState {
    let uses_migrations_table = content.contains("migrations_table");
    let mut mods = Vec::new();
    for line in content.lines() {
        if let Some(rest) = line.trim().strip_prefix("mod ")
            && let Some(name) = rest.strip_suffix(';')
            && name != SEA_ORM_CLI_PLACEHOLDER_MODULE
            && !mods.contains(&name.to_string())
        {
            mods.push(name.to_string());
        }
    }
    let entries = match find_vec_span(content) {
        Some((s, e)) => extract_box_entries(&content[s..e])
            .into_iter()
            .filter(|entry| !entry.contains(SEA_ORM_CLI_PLACEHOLDER_MODULE))
            .collect(),
        None => Vec::new(),
    };
    LibState {
        uses_migrations_table,
        mods,
        entries,
    }
}

fn render_lib(state: &LibState) -> String {
    let mut out = String::new();
    if state.uses_migrations_table {
        out.push_str("use runique::prelude::migrations_table;\n");
    }
    out.push_str("use sea_orm_migration::prelude::*;\n");
    for m in &state.mods {
        out.push_str(&format!("mod {};\n", m));
    }
    out.push('\n');
    out.push_str("pub struct Migrator;\n\n");
    out.push_str("#[async_trait::async_trait]\n");
    out.push_str("impl MigratorTrait for Migrator {\n");
    out.push_str("    fn migrations() -> Vec<Box<dyn MigrationTrait>> {\n        ");
    if state.entries.is_empty() {
        out.push_str("vec![]");
    } else {
        out.push_str("vec![\n");
        for e in &state.entries {
            out.push_str("            ");
            out.push_str(e);
            out.push_str(",\n");
        }
        out.push_str("        ]");
    }
    out.push_str("\n    }\n}\n");
    out
}

/// Deletes the `sea-orm-cli migrate init` placeholder migration file, if present.
fn strip_sea_orm_cli_placeholder_file(migrations_path: &str) {
    let placeholder_file = format!("{}/{}.rs", migrations_path, SEA_ORM_CLI_PLACEHOLDER_MODULE);
    if Path::new(&placeholder_file).exists() {
        let _ = fs::remove_file(&placeholder_file);
    }
}

// ── topological sort ─────────────────────────────────────────────────────────

/// Sorts `Changes` by FK dependency order: a new table comes after the new
/// tables it references, ties broken by name so the order never depends on a
/// hash map. A cycle of new tables is broken at its smallest name; the foreign
/// keys left pointing forward are added once their target exists (see
/// `build_main_plan`). Changes to existing tables follow, by name.
pub(crate) fn topological_sort_changes(
    changes: Vec<crate::migration::utils::types::Changes>,
) -> Vec<crate::migration::utils::types::Changes> {
    use std::collections::{BTreeMap, BTreeSet};

    let new_tables: BTreeSet<String> = changes
        .iter()
        .filter(|c| c.is_new_table)
        .map(|c| c.table_name.clone())
        .collect();

    // waits_for[A] = new tables A references (not yet placed)
    let mut waits_for: BTreeMap<String, BTreeSet<String>> = new_tables
        .iter()
        .map(|t| (t.clone(), BTreeSet::new()))
        .collect();
    for change in changes.iter().filter(|c| c.is_new_table) {
        for fk in &change.added_fks {
            if new_tables.contains(&fk.to_table) && fk.to_table != change.table_name {
                waits_for
                    .entry(change.table_name.clone())
                    .or_default()
                    .insert(fk.to_table.clone());
            }
        }
    }

    let mut sorted_names: Vec<String> = Vec::with_capacity(new_tables.len());
    while !waits_for.is_empty() {
        let next = waits_for
            .iter()
            .find(|(_, deps)| deps.is_empty())
            .or_else(|| waits_for.iter().next())
            .map(|(t, _)| t.clone())
            .expect("waits_for is not empty");
        waits_for.remove(&next);
        for deps in waits_for.values_mut() {
            deps.remove(&next);
        }
        sorted_names.push(next);
    }

    let mut by_name: BTreeMap<String, crate::migration::utils::types::Changes> = changes
        .into_iter()
        .map(|c| (c.table_name.clone(), c))
        .collect();
    let mut result: Vec<crate::migration::utils::types::Changes> =
        Vec::with_capacity(by_name.len());
    for name in sorted_names {
        if let Some(c) = by_name.remove(&name) {
            result.push(c);
        }
    }
    result.extend(by_name.into_values());
    result
}

// ── destructive change guard ─────────────────────────────────────────────────

/// Scans a set of changes for destructive operations (dropped columns, type
/// changes, nullable-to-required, dropped FKs, new `ON DELETE CASCADE` on an
/// existing table) and returns one human-readable warning per finding. An
/// empty result means the changes are safe to apply without `--force`.
pub fn collect_destructive_messages(all_changes: &[Changes]) -> Vec<String> {
    let dropped = all_changes.iter().flat_map(|c| {
        c.dropped_columns
            .iter()
            .map(|col| format!("  {}.{}: DROP COLUMN (data loss)", c.table_name, col.name))
    });

    let type_changes = all_changes.iter().flat_map(|c| {
        c.modified_columns
            .iter()
            // `col_type` alone misses becoming (or stopping being) an enum — an enum
            // column keeps col_type "String", the shape change lives in
            // `enum_string_values` instead. Without this, a string->enum column would
            // generate silently (no `--force` prompt) with only a code comment warning
            // the developer might never see.
            .filter(|(old, new)| {
                old.col_type != new.col_type
                    || old.enum_string_values.is_empty() != new.enum_string_values.is_empty()
            })
            .map(|(old, new)| {
                if old.col_type != new.col_type {
                    format!(
                        "  {}.{}: type {} -> {}",
                        c.table_name, old.name, old.col_type, new.col_type
                    )
                } else if new.enum_string_values.is_empty() {
                    format!(
                        "  {}.{}: enum -> {} (manual migration required)",
                        c.table_name, old.name, new.col_type
                    )
                } else {
                    format!(
                        "  {}.{}: {} -> enum (manual migration required)",
                        c.table_name, old.name, old.col_type
                    )
                }
            })
    });

    let nullable_to_required = all_changes.iter().flat_map(|c| {
        c.modified_columns
            .iter()
            .filter(|(old, new)| old.nullable && !new.nullable && old.col_type == new.col_type)
            .map(|(_, new)| {
                format!(
                    "  {}.{}: nullable -> not_null (requires a default or backfill)",
                    c.table_name, new.name
                )
            })
    });

    let length_shrinks = all_changes.iter().flat_map(|c| {
        c.modified_columns
            .iter()
            .filter(|(old, new)| {
                old.col_type == new.col_type && length_may_shrink(old.max_length, new.max_length)
            })
            .map(|(old, new)| {
                let len =
                    |l: Option<u32>| l.map_or_else(|| "default".to_string(), |n| n.to_string());
                format!(
                    "  {}.{}: length {} -> {} (longer values would be cut or refused)",
                    c.table_name,
                    new.name,
                    len(old.max_length),
                    len(new.max_length)
                )
            })
    });

    let dropped_fks = all_changes.iter().flat_map(|c| {
        c.dropped_fks.iter().map(|fk| {
            format!(
                "  {}.{}: DROP FOREIGN KEY -> {} (orphan records possible)",
                c.table_name, fk.from_column, fk.to_table
            )
        })
    });

    // Adding a CASCADE constraint to existing data can trigger mass deletes if a parent is removed.
    // Only a concern on existing tables: a table created in this same batch has no rows yet,
    // so a CASCADE FK on a brand-new table carries no data-loss risk (avoids a false positive
    // on first generation / from-scratch regeneration).
    let cascade_fks = all_changes
        .iter()
        .filter(|c| !c.is_new_table)
        .flat_map(|c| {
            c.added_fks
                .iter()
                .filter(|fk| fk.on_delete.to_uppercase() == "CASCADE")
                .map(|fk| {
                    format!(
                        "  {}.{}: ADD FOREIGN KEY -> {} ON DELETE CASCADE (existing rows may be deleted)",
                        c.table_name, fk.from_column, fk.to_table
                    )
                })
        });

    dropped
        .chain(type_changes)
        .chain(nullable_to_required)
        .chain(length_shrinks)
        .chain(dropped_fks)
        .chain(cascade_fks)
        .collect()
}

/// Whether going from `old` to `new` column length can drop data on some
/// engine. No length is unbounded on Postgres but `VARCHAR(255)` on MySQL, so
/// both directions out of "no length" are checked against each.
fn length_may_shrink(old: Option<u32>, new: Option<u32>) -> bool {
    match (old, new) {
        (Some(old), Some(new)) => new < old,
        (None, Some(_)) => true,
        (Some(old), None) => old > 255,
        (None, None) => false,
    }
}

/// Prints `messages` under the translated `header_key`, then bails with the translated
/// `bail_key` — the shared "list every violation, then stop" shape behind both
/// `check_destructive` (forceable) and `check_identifier_lengths` (never forceable, always
/// passes `force: false`). An empty `messages` or `force: true` is a silent no-op.
fn report_and_bail_if_any(
    messages: &[String],
    header_key: &str,
    bail_key: &str,
    force: bool,
) -> Result<()> {
    if messages.is_empty() || force {
        return Ok(());
    }

    eprintln!("\n{}", t(header_key));
    for msg in messages {
        eprintln!("{}", msg);
    }
    anyhow::bail!("{}", t(bail_key));
}

fn check_destructive(all_changes: &[Changes], force: bool) -> Result<()> {
    let blocking = collect_destructive_messages(all_changes);
    report_and_bail_if_any(
        &blocking,
        "makemigrations.destructive_detected",
        "makemigrations.destructive_require_force",
        force,
    )
}

// ── generated-identifier length guard ──────────────────────────────────────────

/// Scans a set of changes for FK constraint / index names that would exceed
/// MariaDB/MySQL's 64-character identifier limit (63 used here as the shared safe bound —
/// Postgres's own NAMEDATALEN-1 limit is one byte tighter). Checked once, upfront, over
/// the whole plan — before any migration file is generated — so the string-building
/// generators (`generators.rs`) and the DSL-to-schema conversion (`to_schema.rs`) stay
/// pure and infallible. There is no `--force` escape hatch here, unlike
/// `check_destructive`: Runique won't guess a shortened name for you (that would make it
/// unpredictable from the `model!{}` declaration alone), so a human has to rename
/// something regardless.
fn collect_long_identifier_messages(all_changes: &[Changes]) -> Vec<String> {
    const MAX_LEN: usize = 63;
    let mut messages = Vec::new();

    for change in all_changes {
        for fk in change.added_fks.iter().chain(change.dropped_fks.iter()) {
            let name = format!(
                "{}_{}_{}_fkey",
                change.table_name, fk.from_column, fk.to_table
            );
            if name.len() > MAX_LEN {
                messages.push(format!(
                    "  {name} ({len} characters, max {MAX_LEN}) — shorten the table name, '{col}', or the target table name",
                    len = name.len(),
                    col = fk.from_column,
                ));
            }
        }
        for idx in change
            .added_indexes
            .iter()
            .chain(change.dropped_indexes.iter())
        {
            if idx.name.len() > MAX_LEN {
                messages.push(format!(
                    "  {} ({} characters, max {MAX_LEN}) — shorten the table or column names in this index",
                    idx.name,
                    idx.name.len(),
                ));
            }
        }
    }
    messages
}

fn check_identifier_lengths(all_changes: &[Changes]) -> Result<()> {
    let long_ids = collect_long_identifier_messages(all_changes);
    report_and_bail_if_any(
        &long_ids,
        "makemigrations.long_identifier_detected",
        "makemigrations.long_identifier_rename_required",
        false,
    )
}

// ── run ──────────────────────────────────────────────────────────────────────
/// Builds the SeaORM module name for an ALTER migration on `table` (`m{timestamp}_alter_{table}_table`).
pub fn seaorm_alter_module_name(timestamp: &str, table: &str) -> String {
    format!("m{}_alter_{}_table", timestamp, table)
}

/// Builds the file path of the SeaORM ALTER migration for `table`.
pub fn seaorm_alter_file_path(migrations_path: &str, timestamp: &str, table: &str) -> String {
    format!(
        "{}/{}.rs",
        migrations_path,
        seaorm_alter_module_name(timestamp, table)
    )
}

/// Builds the SeaORM module name for an `extend!{}` migration on `table` (`m{timestamp}_extend_{table}_table`).
pub fn seaorm_extend_module_name(timestamp: &str, table: &str) -> String {
    format!("m{}_extend_{}_table", timestamp, table)
}

/// Builds the file path of the SeaORM `extend!{}` migration for `table`.
pub fn seaorm_extend_file_path(migrations_path: &str, timestamp: &str, table: &str) -> String {
    format!(
        "{}/m{}_extend_{}_table.rs",
        migrations_path, timestamp, table
    )
}

// ── scan extend blocks ────────────────────────────────────────────────────────

/// Scans all `.rs` files in the `entities_path` directory and collects
/// all found `extend!{}` blocks. Returns a flat list of `ParsedSchema`
/// (one per block — multiple blocks can target the same table).
pub fn scan_extend_blocks(entities_path: &str) -> Result<Vec<ParsedSchema>> {
    let mut schemas = Vec::new();
    let entries = fs::read_dir(entities_path)
        .with_context(|| format!("Cannot read entities directory: {}", entities_path))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        if path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
            continue;
        }

        let source = fs::read_to_string(&path)
            .with_context(|| format!("Cannot read file: {}", path.display()))?;

        let blocks = parse_extend_blocks_from_source(&source)
            .with_context(|| format!("Invalid extend!{{}} in {}", path.display()))?;
        for schema in blocks {
            schemas.push(schema);
        }
    }
    Ok(schemas)
}

/// Merges `extend!{}` blocks targeting the same table into a single `ParsedSchema`.
/// Columns are concatenated in discovery order.
pub fn merge_extend_schemas(schemas: Vec<ParsedSchema>) -> Vec<ParsedSchema> {
    use std::collections::HashMap;
    let mut by_table: HashMap<String, Vec<ParsedColumn>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();

    for schema in schemas {
        if !by_table.contains_key(&schema.table_name) {
            order.push(schema.table_name.clone());
            by_table.insert(schema.table_name.clone(), Vec::new());
        }
        by_table
            .get_mut(&schema.table_name)
            .unwrap()
            .extend(schema.columns);
    }

    order
        .into_iter()
        .map(|table_name| {
            let columns = by_table.remove(&table_name).unwrap_or_default();
            ParsedSchema {
                table_name,
                primary_key: None,
                columns,
                foreign_keys: Vec::new(),
                indexes: Vec::new(),
            }
        })
        .collect()
}
/// Entry point of the `makemigrations` command: scans models and `extend!{}`
/// blocks, diffs them against their last snapshot, blocks on destructive
/// changes unless `force` is set, then writes the full migration plan atomically.
pub fn run(entities_path: &str, migrations_path: &str, force: bool) -> Result<()> {
    let schemas = scan_entities(entities_path)?;

    fs::create_dir_all(snapshot_dir(migrations_path))?;

    // ── Plan everything up front — nothing is written until the full plan
    //    (main models + extend!{} blocks) is computed and validated.
    let mut main_changes = compute_main_changes(&schemas, migrations_path)?;
    let extend_planned = plan_extend_changes(entities_path, migrations_path)?;

    let upgrades = snapshot_upgrades(
        entities_path,
        &schemas,
        &main_changes,
        &extend_planned,
        migrations_path,
    )?;

    if main_changes.is_empty() && extend_planned.is_empty() {
        for (path, content) in &upgrades {
            fs::write(path, content)?;
        }
        if !upgrades.is_empty() {
            println!(
                "{}",
                tf("makemigrations.snapshots_upgraded", &[upgrades.len()])
            );
        }
        return Ok(());
    }

    // ── Single destructive guard over main + extend (honors --force) ──────
    let mut destructive_set: Vec<Changes> = main_changes.clone();
    destructive_set.extend(extend_planned.iter().map(|(_, c)| c.clone()));
    check_destructive(&destructive_set, force)?;
    check_identifier_lengths(&destructive_set)?;

    let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();

    // referenced tables created before those referencing them
    main_changes = topological_sort_changes(main_changes);

    // ── Build a single unified plan (no writing) ──────────────────────────
    let mut plan = Plan::default();
    build_main_plan(
        &mut plan,
        &main_changes,
        &schemas,
        migrations_path,
        &timestamp,
    );
    build_extend_plan(&mut plan, &extend_planned, migrations_path, &timestamp);
    plan.files.extend(upgrades);

    // ── One atomic commit: dirs → backups → write → lib.rs → admin positioning,
    //    with a single rollback covering all of it.
    let module_count = plan.lib_modules.len();
    commit_plan(&plan, migrations_path)?;

    println!("{}", tf("makemigrations.files_ready", &[module_count]));

    Ok(())
}

// ── unified plan ───────────────────────────────────────────────────────────────

/// A fully-computed migration plan: everything to create/write/register, no side effects.
/// Built before any IO so the destructive guard and a single atomic commit can run on it.
#[derive(Default)]
struct Plan {
    /// (path, content) files to write
    files: Vec<(String, String)>,
    /// directories to create before writing
    dirs: Vec<String>,
    /// migration modules to register in `lib.rs`, in order
    lib_modules: Vec<String>,
}

/// Computes the diff for every scanned model (no writing).
fn compute_main_changes(schemas: &[ParsedSchema], migrations_path: &str) -> Result<Vec<Changes>> {
    let mut all_changes: Vec<Changes> = Vec::new();
    for schema in schemas {
        let snap_path = snapshot_file_path(migrations_path, &schema.table_name);
        let changes = if Path::new(&snap_path).exists() {
            let previous = previous_snapshot(&snap_path, schema)?;
            diff_schemas(&previous, schema)
        } else {
            Changes {
                table_name: schema.table_name.clone(),
                added_columns: db_columns(schema).into_iter().cloned().collect(),
                dropped_columns: vec![],
                modified_columns: vec![],
                renamed_columns: vec![],
                added_fks: schema.foreign_keys.clone(),
                dropped_fks: vec![],
                added_indexes: schema.indexes.clone(),
                dropped_indexes: vec![],
                is_new_table: true,
                enum_renames: vec![],
                enum_value_adds: vec![],
                enum_value_drops: vec![],
            }
        };
        if !changes.is_empty() {
            all_changes.push(changes);
        }
    }
    Ok(all_changes)
}

/// For each new table, the foreign keys of an FK cycle: keys to a new table
/// created later in this batch (`forward`), and keys pointing to it from a
/// table created earlier (`closing`). `all_changes` is in creation order.
fn cycle_keys<'a>(
    all_changes: &[Changes],
    schemas: &'a [ParsedSchema],
) -> std::collections::HashMap<&'a str, CycleKeys<'a>> {
    use std::collections::{HashMap, HashSet};

    let new_tables: HashSet<&str> = all_changes
        .iter()
        .filter(|c| c.is_new_table)
        .map(|c| c.table_name.as_str())
        .collect();
    let mut created: HashSet<&str> = HashSet::new();
    let mut keys: HashMap<&'a str, CycleKeys<'a>> = HashMap::new();
    for change in all_changes.iter().filter(|c| c.is_new_table) {
        let Some(schema) = schemas.iter().find(|s| s.table_name == change.table_name) else {
            continue;
        };
        let table = schema.table_name.as_str();
        for fk in &schema.foreign_keys {
            let target = fk.to_table.as_str();
            if target != table && new_tables.contains(target) && !created.contains(target) {
                keys.entry(table).or_default().forward.push(fk);
                if let Some(target) = schemas.iter().find(|s| s.table_name == target) {
                    keys.entry(target.table_name.as_str())
                        .or_default()
                        .closing
                        .push((table, fk));
                }
            }
        }
        created.insert(table);
    }
    keys
}

/// Adds the main-model migration files/dirs/modules to the plan.
fn build_main_plan(
    plan: &mut Plan,
    all_changes: &[Changes],
    schemas: &[ParsedSchema],
    migrations_path: &str,
    timestamp: &str,
) {
    let cycle_keys = cycle_keys(all_changes, schemas);
    let no_cycle = CycleKeys::default();
    for change in all_changes {
        let schema = schemas
            .iter()
            .find(|s| s.table_name == change.table_name)
            .unwrap();

        // Snapshot — includes FK stmts so future diffs detect FK additions/removals.
        plan.files.push((
            snapshot_file_path(migrations_path, &change.table_name),
            generate_snapshot_file(schema),
        ));

        if change.is_new_table {
            let module_name = seaorm_create_module_name(timestamp, &change.table_name);
            let seaorm_path =
                seaorm_create_file_path(migrations_path, timestamp, &change.table_name);
            let keys = cycle_keys
                .get(change.table_name.as_str())
                .unwrap_or(&no_cycle);
            plan.files
                .push((seaorm_path, generate_create_file_in_cycle(schema, keys)));
            plan.lib_modules.push(module_name);
        } else {
            let module_name = seaorm_alter_module_name(timestamp, &change.table_name);
            let seaorm_path =
                seaorm_alter_file_path(migrations_path, timestamp, &change.table_name);
            plan.files.push((seaorm_path, generate_alter_file(change)));
            plan.lib_modules.push(module_name);
        }
    }
}

/// Adds the extend!{} migration files/dirs/modules to the plan.
fn build_extend_plan(
    plan: &mut Plan,
    planned: &[(ParsedSchema, Changes)],
    migrations_path: &str,
    timestamp: &str,
) {
    if planned.is_empty() {
        return;
    }
    plan.dirs.push(extend_snapshot_dir(migrations_path));

    for (ext_schema, changes) in planned {
        // Snapshot updated (without PK, just extension columns)
        plan.files.push((
            extend_snapshot_file_path(migrations_path, &ext_schema.table_name),
            extend_snapshot_content(ext_schema),
        ));

        let module_name = seaorm_extend_module_name(timestamp, &ext_schema.table_name);
        let seaorm_path =
            seaorm_extend_file_path(migrations_path, timestamp, &ext_schema.table_name);
        plan.files.push((seaorm_path, generate_alter_file(changes)));
        plan.lib_modules.push(module_name);
    }
}

/// Writes the whole plan atomically: create dirs, back up existing targets, write files,
/// register lib.rs modules, position AdminTableMigration — all under a single rollback.
fn commit_plan(plan: &Plan, migrations_path: &str) -> Result<()> {
    // Directory creation (idempotent)
    for dir in &plan.dirs {
        fs::create_dir_all(dir)?;
    }

    // lib.rs backup for rollback in case of partial error.
    let lib_file = lib_path(migrations_path);
    let lib_backup: Option<String> = if Path::new(&lib_file).exists() {
        Some(fs::read_to_string(&lib_file)?)
    } else {
        None
    };

    // Back up the previous content of any target that already exists (snapshots especially),
    // so the rollback restores it instead of deleting it — a deleted snapshot would make the
    // next run regenerate a full CREATE for an already-migrated table.
    let mut file_backups: StrMap = StrMap::new();
    for (path, _) in &plan.files {
        if Path::new(path).exists()
            && let Ok(prev) = fs::read_to_string(path)
        {
            file_backups.insert(path.clone(), prev);
        }
    }

    let mut written: Vec<String> = Vec::new();
    let write_result: Result<()> = (|| {
        for (path, content) in &plan.files {
            fs::write(path, content).with_context(|| format!("Failed to write: {}", path))?;
            written.push(path.clone());
        }
        for module_name in &plan.lib_modules {
            update_migration_lib(migrations_path, module_name)?;
        }
        // AdminTableMigration positioning rewrites lib.rs — kept inside the protected
        // scope so a failure here also triggers the rollback below.
        ensure_admin_migration_positioned(migrations_path)?;
        Ok(())
    })();

    if let Err(e) = write_result {
        eprintln!(
            "\n[makemigrations] Error: {}. Rollback generated files...",
            e
        );
        for path in &written {
            match file_backups.get(path) {
                Some(prev) => {
                    if let Err(re) = fs::write(path, prev) {
                        eprintln!("  warning: cannot restore {} : {}", path, re);
                    } else {
                        eprintln!("  restored: {}", path);
                    }
                }
                None => {
                    if let Err(re) = fs::remove_file(path) {
                        eprintln!("  warning: cannot delete {} : {}", path, re);
                    } else {
                        eprintln!("  deleted: {}", path);
                    }
                }
            }
        }
        match lib_backup {
            Some(content) => {
                let _ = fs::write(&lib_file, content);
                eprintln!("  lib.rs restored");
            }
            None => {
                let _ = fs::remove_file(&lib_file);
            }
        }
        return Err(e);
    }

    Ok(())
}

// ── AdminTableMigration positioning ───────────────────────────────────────

/// Puts the framework migrations (`eihwaz_users`, sessions, admin tables,
/// reset tokens) at the top of `lib.rs`, before the app's own.
///
/// - Adds `use runique::prelude::migrations_table;` if missing
/// - Drops app migrations that would recreate a framework table
/// - No effect when `lib.rs` doesn't exist yet
pub fn ensure_admin_migration_positioned(migrations_path: &str) -> Result<()> {
    let lib_file = lib_path(migrations_path);

    if !Path::new(&lib_file).exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&lib_file)?;
    strip_sea_orm_cli_placeholder_file(migrations_path);
    let mut state = parse_lib_state(&content);
    state.uses_migrations_table = true;

    let admin_box = "Box::new(migrations_table::AdminTableMigration)".to_string();
    let sessions_box = "Box::new(migrations_table::EihwazSessionsMigration)".to_string();
    let reset_box = "Box::new(migrations_table::EihwazResetTokensMigration)".to_string();
    let users_box = "Box::new(migrations_table::EihwazUsersMigration)".to_string();

    // Tables created by `EihwazUsersMigration` + `EihwazSessionsMigration` + `AdminTableMigration` — to exclude from the vec
    const FRAMEWORK_TABLE_PATTERNS: &[&str] = &[
        "create_eihwaz_users_table",
        "create_eihwaz_groupes_table",
        "create_eihwaz_groupes_droits_table",
        "create_eihwaz_users_groupes_table",
        "create_eihwaz_sessions_table",
        "create_eihwaz_reset_tokens_table",
    ];

    // Remove existing framework entries/mods (we'll re-inject at the top)
    // and also drop app migrations duplicating framework tables.
    state
        .entries
        .retain(|e| e != &users_box && e != &sessions_box && e != &reset_box && e != &admin_box);
    state
        .mods
        .retain(|m| !FRAMEWORK_TABLE_PATTERNS.iter().any(|pat| m.contains(pat)));
    state.entries.splice(
        0..0,
        [
            users_box.clone(),
            sessions_box.clone(),
            admin_box.clone(),
            reset_box.clone(),
        ],
    );

    let result = render_lib(&state);
    if result != content {
        fs::write(&lib_file, &result)?;
    }

    Ok(())
}

// ── Extend pass (planning) ─────────────────────────────────────────────────────

/// Scans + merges `extend!{}` blocks and computes their diffs (no writing).
/// Returns the owned schema + change pair for each table that actually changed.
fn plan_extend_changes(
    entities_path: &str,
    migrations_path: &str,
) -> Result<Vec<(ParsedSchema, Changes)>> {
    let raw_extends = scan_extend_blocks(entities_path)?;
    if raw_extends.is_empty() {
        return Ok(Vec::new());
    }

    let extend_schemas = merge_extend_schemas(raw_extends);

    let mut planned: Vec<(ParsedSchema, Changes)> = Vec::new();
    for ext_schema in extend_schemas {
        let snap_path = extend_snapshot_file_path(migrations_path, &ext_schema.table_name);

        let changes = if Path::new(&snap_path).exists() {
            let previous = previous_snapshot(&snap_path, &ext_schema)?;
            diff_schemas(&previous, &ext_schema)
        } else {
            // First time — all columns are new (ADD COLUMN)
            Changes {
                table_name: ext_schema.table_name.clone(),
                added_columns: ext_schema.columns.clone(),
                dropped_columns: vec![],
                modified_columns: vec![],
                renamed_columns: vec![],
                added_fks: vec![],
                dropped_fks: vec![],
                added_indexes: vec![],
                dropped_indexes: vec![],
                is_new_table: false, // always false — the framework table already exists
                enum_renames: vec![],
                enum_value_adds: vec![],
                enum_value_drops: vec![],
            }
        };

        if changes.is_empty() {
            continue;
        }
        planned.push((ext_schema, changes));
    }

    Ok(planned)
}
