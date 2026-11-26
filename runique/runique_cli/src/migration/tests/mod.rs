//! Pure pipeline tests for `makemigrations` — no DB, no Docker.
//!
//! These cover the regressions found during the 2026-06-11 session:
//! - literal `[default: X]` values must be parsed and emitted,
//! - `bool` is a v2 type (nullable unless `required`),
//! - a CASCADE FK on a brand-new table is not a destructive change,
//! - the snapshot round-trip is stable (parse → snapshot → reparse → diff empty),
//! - enum value additions are detected.
//!
//! Laid out by what is tested: `parser`, `diff`, `generate`, `changes`, and
//! `pipeline` for what goes through all of them. What several of them share
//! is here.
mod changes;
mod diff;
mod generate;
mod parser;
mod pipeline;

use super::*;
use crate::migration::steps::changes::{collect_destructive_messages, topological_sort_changes};

fn parse_model(src: &str) -> ParsedSchema {
    parse_schema_from_source(src)
        .unwrap()
        .expect("model! source should parse")
        .1
}

fn col<'a>(schema: &'a ParsedSchema, name: &str) -> &'a ParsedColumn {
    schema
        .columns
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("column `{name}` not found"))
}

/// Minimal `Changes` for destructive-guard tests.
fn change(
    is_new_table: bool,
    added_fks: Vec<ParsedFk>,
    dropped_columns: Vec<ParsedColumn>,
) -> Changes {
    Changes {
        table_name: "t".into(),
        dropped_columns,
        added_fks,
        table: if is_new_table {
            crate::migration::TableChange::Created
        } else {
            crate::migration::TableChange::Altered
        },
        ..Default::default()
    }
}

fn cascade_fk() -> ParsedFk {
    ParsedFk {
        from_column: "parent_id".into(),
        to_table: "parent".into(),
        to_column: "id".into(),
        on_delete: crate::migration::FkAction::Cascade,
        on_update: crate::migration::FkAction::NoAction,
    }
}

const BLOG_SRC: &str = r#"
use runique::prelude::*;
model! {
    Blog,
    table: "blog",
    pk: id => Pk,
    {
        title:      text [required],
        view_count: int  [default: 0],
        is_active:  bool [default: true],
    }
}
"#;

fn empty_changes() -> Changes {
    Changes {
        table_name: "t".into(),
        ..Default::default()
    }
}

fn up_down(sql: &str) -> (&str, &str) {
    let mut parts = sql.split("async fn down");
    let up = parts.next().unwrap_or("");
    let down = parts.next().unwrap_or("");
    (up, down)
}

fn fk(from: &str, to_table: &str) -> ParsedFk {
    ParsedFk {
        from_column: from.into(),
        to_table: to_table.into(),
        to_column: "id".into(),
        on_delete: crate::migration::FkAction::NoAction,
        on_update: crate::migration::FkAction::NoAction,
    }
}

const TYPES_MODEL: &str = r#"
use runique::prelude::*;
model! {
    Mix,
    table: "mix",
    pk: id => Pk,
    {
        price:   decimal [nullable, max_digits: 10, decimal_places: 2],
        active:  bool [nullable],
        ref_id:  uuid [nullable],
        payload: json [nullable],
        big:     bigint [nullable],
        ratio:   float [nullable],
    }
}
"#;

/// `person` with `field` and an `age` column nothing touches.
fn person(field: &str) -> ParsedSchema {
    parse_model(&format!(
        r#"model! {{ Person, table: "person", pk: id => i32, {{ {field}, age: int [nullable] }} }}"#
    ))
}
