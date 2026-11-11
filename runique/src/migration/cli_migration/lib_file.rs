//! The migration crate's `lib.rs`: parsing, canonical rendering, framework migrations first.
use crate::migration::*;
use anyhow::Result;
use std::fs;
use std::path::Path;

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
pub(super) const SEA_ORM_CLI_PLACEHOLDER_MODULE: &str = "m20220101_000001_create_table";

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
pub(super) fn find_vec_span(content: &str) -> Option<(usize, usize)> {
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
pub(super) fn extract_box_entries(vec_content: &str) -> Vec<String> {
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
pub(super) fn strip_sea_orm_cli_placeholder_file(migrations_path: &str) {
    let placeholder_file = format!("{}/{}.rs", migrations_path, SEA_ORM_CLI_PLACEHOLDER_MODULE);
    if Path::new(&placeholder_file).exists() {
        let _ = fs::remove_file(&placeholder_file);
    }
}

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
    position_framework_migrations(&mut state);

    let result = render_lib(&state);
    if result != content {
        fs::write(&lib_file, &result)?;
    }

    Ok(())
}

/// The framework's migrations, in the order they must run.
const FRAMEWORK_MIGRATIONS: [&str; 5] = [
    "Box::new(migrations_table::EihwazUsersMigration)",
    "Box::new(migrations_table::EihwazSessionsMigration)",
    "Box::new(migrations_table::AdminTableMigration)",
    "Box::new(migrations_table::EihwazHistoryMigration)",
    "Box::new(migrations_table::EihwazResetTokensMigration)",
];

/// The `lib.rs` of a migration crate with no model of its own yet: the
/// framework's tables only. `runique new` writes it; `makemigrations` would
/// produce the very same file.
pub(crate) fn initial_lib() -> String {
    let mut state = LibState::default();
    position_framework_migrations(&mut state);
    render_lib(&state)
}

fn position_framework_migrations(state: &mut LibState) {
    state.uses_migrations_table = true;

    // Tables created by `EihwazUsersMigration` + `EihwazSessionsMigration` + `AdminTableMigration` — to exclude from the vec
    const FRAMEWORK_TABLE_PATTERNS: &[&str] = &[
        "create_eihwaz_users_table",
        "create_eihwaz_groupes_table",
        "create_eihwaz_groupes_droits_table",
        "create_eihwaz_users_groupes_table",
        "create_eihwaz_sessions_table",
        "create_eihwaz_reset_tokens_table",
        "create_eihwaz_history_table",
    ];

    // Remove existing framework entries/mods (we'll re-inject at the top)
    // and also drop app migrations duplicating framework tables.
    state
        .entries
        .retain(|e| !FRAMEWORK_MIGRATIONS.contains(&e.as_str()));
    state
        .mods
        .retain(|m| !FRAMEWORK_TABLE_PATTERNS.iter().any(|pat| m.contains(pat)));
    state
        .entries
        .splice(0..0, FRAMEWORK_MIGRATIONS.map(String::from));
}
