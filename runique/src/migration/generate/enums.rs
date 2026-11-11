//! Postgres enum types: creating and dropping them, and moving a column to or from an enum.
use super::columns::{col_type_method, enum_column_type};
use crate::migration::types::{ParsedColumn, ParsedSchema};

// Every function below writes a `CREATE TYPE`/`DROP TYPE` — Postgres-only syntax —
// unconditionally into the generated migration, wrapped in a runtime
// `manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres`
// check rather than gated by `db_kind` at generation time. A migration file is
// fixed forever once written; deciding "include this or not" from whatever engine
// `makemigrations` happened to target freezes the file to that ONE engine
// (confirmed in practice: a Postgres-generated `CREATE TYPE ... AS ENUM` run against
// MariaDB is a straight syntax error). Checking the real backend at migration-RUN
// time instead makes the same generated file portable across all three.

pub(super) fn build_enum_type_stmts(schema: &ParsedSchema) -> String {
    build_enum_create_stmts_for_cols(&schema.columns)
}

pub(super) fn build_enum_type_drops(schema: &ParsedSchema) -> String {
    build_enum_drop_stmts_for_cols(&schema.columns)
}

pub(super) fn build_enum_create_stmts_for_cols(cols: &[ParsedColumn]) -> String {
    cols.iter()
        .filter(|col| !col.enum_string_values.is_empty())
        .map(|col| {
            enum_create_stmt(
                col.enum_name.as_deref().unwrap_or(&col.name),
                &col.enum_string_values,
            )
        })
        .collect()
}

pub(super) fn build_enum_drop_stmts_for_cols(cols: &[ParsedColumn]) -> String {
    cols.iter()
        .filter(|col| !col.enum_string_values.is_empty())
        .map(|col| enum_drop_stmt(col.enum_name.as_deref().unwrap_or(&col.name)))
        .collect()
}

/// Creates the Postgres enum type `name`, unless it exists already — Postgres
/// has no `CREATE TYPE IF NOT EXISTS`, so the catalogue is looked up first (an
/// enum shared by several tables is created by the first migration only).
///
/// The name is lowercased: the builder quotes type names, and every other
/// reference to the type (`ColumnType::Enum`, `Type::alter()` below) uses the
/// lowercase form.
fn enum_create_stmt(name: &str, values: &[String]) -> String {
    let name = name.to_lowercase();
    let values = values
        .iter()
        .map(|v| format!("Alias::new({v:?})"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {{\n            let exists = manager\n                .get_connection()\n                .query_one(\n                    &Query::select()\n                        .expr(Expr::val(1))\n                        .from((Alias::new(\"pg_catalog\"), Alias::new(\"pg_type\")))\n                        .and_where(Expr::col(Alias::new(\"typname\")).eq({name:?}))\n                        .to_owned(),\n                )\n                .await?\n                .is_some();\n            if !exists {{\n                manager\n                    .create_type(\n                        sea_query::extension::postgres::Type::create()\n                            .as_enum(Alias::new({name:?}))\n                            .values([{values}])\n                            .to_owned(),\n                    )\n                    .await?;\n            }}\n        }}\n\n"
    )
}

/// Drops the Postgres enum type `name` if it exists.
fn enum_drop_stmt(name: &str) -> String {
    let name = name.to_lowercase();
    format!(
        "        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {{\n            manager\n                .drop_type(\n                    sea_query::extension::postgres::Type::drop()\n                        .if_exists()\n                        .name(Alias::new({name:?}))\n                        .to_owned(),\n                )\n                .await?;\n        }}\n\n"
    )
}

/// A column entering or leaving enum-ness, both directions (`up`: old -> new,
/// `down`: new -> old) — reusing sea-query's builder (`ColumnDef::using`) rather
/// than hand-written SQL for the `ALTER COLUMN ... TYPE ... USING` cast: bare
/// `modify_column` renders `ALTER COLUMN x TYPE enum_type` with no `USING`
/// clause at all, which Postgres rejects on an existing non-enum column (it
/// requires an explicit cast to know how to reinterpret each row's data).
/// `.using()` is Postgres-specific — the other backends' `TableAlterOption`
/// renderers never read `column_def.spec.using`, so setting it unconditionally
/// is harmless there; MySQL's `ENUM(...)` is inline (no separate type to create
/// first) and SQLite has no native enum (plain column either way).
pub(super) fn build_enum_transition(
    table: &str,
    old: &ParsedColumn,
    new: &ParsedColumn,
) -> (String, String) {
    let up = render_enum_column_change(table, old, new);
    let down = render_enum_column_change(table, new, old);
    (up, down)
}

/// Renders one direction of an enum-transition ALTER: `from` is the column's
/// current shape, `to` is what it becomes after this statement runs.
fn render_enum_column_change(table: &str, from: &ParsedColumn, to: &ParsedColumn) -> String {
    let mut out = String::new();
    let from_is_enum = !from.enum_string_values.is_empty();
    let to_is_enum = !to.enum_string_values.is_empty();

    // The enum TYPE must exist before any column can be cast into it — Postgres only
    // (MySQL's enum is inline on the column, SQLite has no native enum type at all),
    // checked at migration-run time rather than baked in at generation time (same
    // reasoning as the enum CREATE/DROP helpers above).
    if to_is_enum && !from_is_enum {
        let enum_name = to.enum_name.as_deref().unwrap_or(&to.name);
        out.push_str(&enum_create_stmt(enum_name, &to.enum_string_values));
    }

    let col_def = if to_is_enum {
        format!(
            "ColumnDef::new_with_type(Alias::new(\"{col}\"), {ty})",
            col = to.name,
            ty = enum_column_type(to),
        )
    } else {
        format!(
            "ColumnDef::new(Alias::new(\"{col}\")).{ty}",
            col = to.name,
            ty = col_type_method(to),
        )
    };
    // Postgres reinterprets each existing row's value through this cast target —
    // the enum name when entering it, `text` when leaving back to a plain column.
    let cast_target = if to_is_enum {
        to.enum_name.as_deref().unwrap_or(&to.name).to_string()
    } else {
        "text".to_string()
    };

    // No `.null()`/`.not_null()` on this modify_column: sea-query's Postgres backend
    // renders `spec.nullable` as a SEPARATE "ALTER COLUMN ... SET/DROP NOT NULL" clause,
    // and `.using()` is appended at the very end of the whole statement regardless of
    // which clause it "belongs" to — combining both produces the invalid
    // "TYPE t, ALTER COLUMN x SET NOT NULL USING expr" (USING stranded after NOT NULL
    // instead of right after TYPE). An enum transition never changes nullability
    // anyway, so the column's existing NOT NULL/NULL constraint is simply left alone.
    // sea-query's SQLite backend `panic!`s unconditionally on ANY `modify_column`
    // (not a SQL-syntax rejection at runtime — the statement builder itself refuses
    // to render one), so this whole ALTER must be skipped there rather than merely
    // guarded like the CREATE/DROP TYPE calls. Harmless to skip: SQLite has no native
    // enum (`ColumnType::Enum` renders as plain `enum_text`), so the column already
    // behaves as free-form text before and after this migration on that engine.
    out.push_str(&format!(
        "        if manager.get_connection().get_database_backend() != sea_orm::DbBackend::Sqlite {{\n            manager\n                .alter_table(\n                    Table::alter()\n                        .table(Alias::new(\"{table}\"))\n                        .modify_column({col_def}.using(Expr::col(Alias::new(\"{col}\")).cast_as(Alias::new(\"{cast_target}\"))))\n                        .to_owned(),\n                )\n                .await?;\n        }}\n\n",
        table = table,
        col_def = col_def,
        col = to.name,
        cast_target = cast_target,
    ));

    // The old type is only droppable once nothing references it any more — after
    // the ALTER above has moved the column off it.
    if from_is_enum && !to_is_enum {
        let enum_name = from.enum_name.as_deref().unwrap_or(&from.name);
        out.push_str(&enum_drop_stmt(enum_name));
    }

    out
}
