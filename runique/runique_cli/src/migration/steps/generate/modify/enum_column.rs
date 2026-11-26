//! A column entering or leaving an enum.
use crate::migration::steps::generate::columns::{col_type_method, enum_column_type};
use crate::migration::steps::generate::{create, delete};
use crate::migration::types::ParsedColumn;

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
///
/// One direction of that change: `from` is the column's current shape, `to`
/// is what it becomes after this statement runs.
pub(crate) fn enum_column(table: &str, from: &ParsedColumn, to: &ParsedColumn) -> String {
    let mut out = String::new();
    let from_is_enum = !from.enum_string_values.is_empty();
    let to_is_enum = !to.enum_string_values.is_empty();

    // The enum TYPE must exist before any column can be cast into it — Postgres only
    // (MySQL's enum is inline on the column, SQLite has no native enum type at all),
    // checked at migration-run time rather than baked in at generation time (same
    // reasoning as in `create/enum_type.rs`).
    if to_is_enum && !from_is_enum {
        let enum_name = to.enum_name.as_deref().unwrap_or(&to.name);
        out.push_str(&create::enum_type(enum_name, &to.enum_string_values));
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
        out.push_str(&delete::enum_type(enum_name));
    }

    out
}
