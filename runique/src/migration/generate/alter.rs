//! The `up`/`down` bodies of an ALTER migration: add, drop, rename and modify columns.
use super::columns::{col_type_method, render_column_def};
use super::constraints::{push_create_fk, push_create_index, push_drop_fk, push_drop_index};
use super::enums::build_enum_transition;
use crate::migration::types::{Changes, ParsedColumn};

/// True if `body` has a line that isn't blank or a `//` comment — i.e. it actually calls `manager`.
/// A type-change diff only emits a `// Manual migration required.` comment, which is non-empty
/// text but no real statement, so a plain `is_empty()` check picks `manager` and leaves it unused.
pub(super) fn body_has_code(body: &str) -> bool {
    body.lines()
        .any(|line| !line.trim().is_empty() && !line.trim_start().starts_with("//"))
}

pub(super) fn build_alter_bodies(change: &Changes) -> (String, String) {
    let mut up = String::new();
    let mut down = String::new();

    // 0) RENAME columns first — later ops (modify/add) may target the new name.
    // `ALTER TABLE … RENAME COLUMN` is portable (PG / MySQL 8+ / MariaDB 10.5+ / SQLite 3.25+).
    for (old, new) in &change.renamed_columns {
        push_rename_column(&mut up, &change.table_name, old, new);
    }

    // 1) DROP indexes (up) / DROP added indexes (down)
    for idx in &change.dropped_indexes {
        push_drop_index(&mut up, &change.table_name, idx);
    }
    for idx in &change.added_indexes {
        push_drop_index(&mut down, &change.table_name, idx);
    }

    // 2) DROP FK
    for fk in &change.dropped_fks {
        push_drop_fk(&mut up, &change.table_name, &fk.from_column, &fk.to_table);
    }
    for fk in &change.added_fks {
        push_drop_fk(&mut down, &change.table_name, &fk.from_column, &fk.to_table);
    }

    // 3) DROP columns
    for col in &change.dropped_columns {
        push_drop_column(&mut up, &change.table_name, &col.name);
    }
    for col in &change.added_columns {
        push_drop_column(&mut down, &change.table_name, &col.name);
    }

    // 4) MODIFY columns
    for (old, new) in &change.modified_columns {
        // Becoming (or stopping being) an enum: `col_type` alone doesn't capture this
        // (an enum column keeps col_type "String"), so this must be checked before the
        // generic "type change => manual" fallback below, which would otherwise catch
        // ordinary type changes but silently miss this one entirely.
        if old.enum_string_values.is_empty() != new.enum_string_values.is_empty() {
            let (up_op, down_op) = build_enum_transition(&change.table_name, old, new);
            up.push_str(&up_op);
            down.push_str(&down_op);
            continue;
        }

        // type change => manual
        if old.col_type != new.col_type {
            up.push_str(&format!(
                "        // WARNING: type change on column '{col}': {old} -> {new}\n        // Manual migration required.\n\n",
                col = new.name,
                old = old.col_type,
                new = new.col_type
            ));
            continue;
        }

        // nullable -> not_null => destructive unless you backfill
        if old.nullable && !new.nullable {
            // Generates modify_column anyway (risky if NULLs exist)
            push_modify_column(&mut up, &change.table_name, new);
            push_modify_column(&mut down, &change.table_name, old);
            continue;
        }

        // safe modify
        push_modify_column(&mut up, &change.table_name, new);
        push_modify_column(&mut down, &change.table_name, old);
    }

    // 5) ADD columns
    for col in &change.added_columns {
        push_add_column(&mut up, &change.table_name, col);
    }

    // 6) Recreate dropped columns in DOWN (now correct because we store ParsedColumn)
    for col in &change.dropped_columns {
        push_add_column(&mut down, &change.table_name, col);
    }

    // 7) ADD FK
    for fk in &change.added_fks {
        push_create_fk(
            &mut up,
            &change.table_name,
            &fk.from_column,
            &fk.to_table,
            &fk.to_column,
            &fk.on_delete,
            &fk.on_update,
        );
    }
    for fk in &change.dropped_fks {
        push_create_fk(
            &mut down,
            &change.table_name,
            &fk.from_column,
            &fk.to_table,
            &fk.to_column,
            &fk.on_delete,
            &fk.on_update,
        );
    }

    // 8) ADD indexes
    for idx in &change.added_indexes {
        push_create_index(&mut up, &change.table_name, idx);
    }
    for idx in &change.dropped_indexes {
        push_create_index(&mut down, &change.table_name, idx);
    }

    // 9) Enum value renames — both branches (Postgres native `RENAME VALUE` vs.
    // VARCHAR-backed `UPDATE`) are always emitted; which one runs is decided at
    // migration-run time via the real backend, not baked in at generation time
    // (a migration file is fixed forever once written — see the enum CREATE/DROP
    // helpers above for the same reasoning).
    // `ALTER TYPE ... RENAME VALUE`/`ADD VALUE` go through
    // `sea_query::extension::postgres::Type`, like the CREATE/DROP TYPE helpers
    // above; the type name is lowercased the same way (the builder quotes it).
    for (col, enum_name, old_val, new_val) in &change.enum_renames {
        let enum_name_lc = enum_name.to_lowercase();
        up.push_str(&format!(
            "        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {{\n            manager\n                .alter_type(\n                    sea_query::extension::postgres::Type::alter()\n                        .name(Alias::new(\"{enum_name_lc}\"))\n                        .rename_value(Alias::new({old_val:?}), Alias::new({new_val:?}))\n                        .to_owned(),\n                )\n                .await?;\n        }} else {{\n            manager\n                .get_connection()\n                .execute(\n                    &Query::update()\n                        .table(Alias::new(\"{table}\"))\n                        .value(Alias::new(\"{col}\"), {new_val:?})\n                        .and_where(Expr::col(Alias::new(\"{col}\")).eq({old_val:?}))\n                        .to_owned(),\n                )\n                .await?;\n        }}\n\n",
            enum_name_lc = enum_name_lc, old_val = old_val, new_val = new_val, table = change.table_name, col = col,
        ));
        down.push_str(&format!(
            "        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {{\n            manager\n                .alter_type(\n                    sea_query::extension::postgres::Type::alter()\n                        .name(Alias::new(\"{enum_name_lc}\"))\n                        .rename_value(Alias::new({new_val:?}), Alias::new({old_val:?}))\n                        .to_owned(),\n                )\n                .await?;\n        }} else {{\n            manager\n                .get_connection()\n                .execute(\n                    &Query::update()\n                        .table(Alias::new(\"{table}\"))\n                        .value(Alias::new(\"{col}\"), {old_val:?})\n                        .and_where(Expr::col(Alias::new(\"{col}\")).eq({new_val:?}))\n                        .to_owned(),\n                )\n                .await?;\n        }}\n\n",
            enum_name_lc = enum_name_lc, old_val = old_val, new_val = new_val, table = change.table_name, col = col,
        ));
    }

    // 10) Enum value additions/removals — Postgres native enum only; VARCHAR-backed
    // enums (non-PG) accept any string, so no DDL is needed there. Guarded at
    // runtime rather than skipped at generation time, same reasoning as above.
    for (_col, enum_name, val) in &change.enum_value_adds {
        let enum_name_lc = enum_name.to_lowercase();
        up.push_str(&format!(
            "        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {{\n            manager\n                .alter_type(\n                    sea_query::extension::postgres::Type::alter()\n                        .name(Alias::new(\"{enum_name_lc}\"))\n                        .add_value(Alias::new({val:?}))\n                        .if_not_exists()\n                        .to_owned(),\n                )\n                .await?;\n        }}\n\n",
            enum_name_lc = enum_name_lc, val = val,
        ));
    }
    for (_col, enum_name, val) in &change.enum_value_drops {
        let enum_name_lc = enum_name.to_lowercase();
        up.push_str(&format!(
            "        // WARNING: value {val:?} removed from {enum_name:?} — manual migration required (Postgres cannot drop an enum value).\n\n",
            val = val, enum_name = enum_name,
        ));
        down.push_str(&format!(
            "        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {{\n            manager\n                .alter_type(\n                    sea_query::extension::postgres::Type::alter()\n                        .name(Alias::new(\"{enum_name_lc}\"))\n                        .add_value(Alias::new({val:?}))\n                        .if_not_exists()\n                        .to_owned(),\n                )\n                .await?;\n        }}\n\n",
            enum_name_lc = enum_name_lc, val = val,
        ));
    }

    // 11) Reverse the column renames last in DOWN (undo of section 0).
    for (old, new) in &change.renamed_columns {
        push_rename_column(&mut down, &change.table_name, new, old);
    }

    (up, down)
}

fn push_rename_column(buf: &mut String, table: &str, from: &str, to: &str) {
    buf.push_str(&format!(
        "        manager\n            .alter_table(\n                Table::alter()\n                    .table(Alias::new(\"{table}\"))\n                    .rename_column(Alias::new(\"{from}\"), Alias::new(\"{to}\"))\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        from = from,
        to = to
    ));
}

fn push_drop_column(buf: &mut String, table: &str, col: &str) {
    buf.push_str(&format!(
        "        manager\n            .alter_table(\n                Table::alter()\n                    .table(Alias::new(\"{table}\"))\n                    .drop_column(Alias::new(\"{col}\"))\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        col = col
    ));
}

fn push_modify_column(buf: &mut String, table: &str, column: &ParsedColumn) {
    let col = &column.name;
    let nullable = column.nullable;
    let unique = column.unique;
    let null = if nullable { ".null()" } else { ".not_null()" };
    let uniq = if unique { ".unique_key()" } else { "" };
    // sea-query's SQLite backend `panic!`s unconditionally on ANY `modify_column`
    // (its ALTER TABLE only ever supports ADD/RENAME/DROP COLUMN — a real SQLite
    // limitation, not a sea-query gap), so this ALTER must be skipped there rather
    // than merely guarded. Unlike the enum-transition case, skipping here genuinely
    // loses the constraint change on SQLite (nullable/unique/type stays whatever it
    // was) — flagged with a warning comment in the generated file rather than
    // silently dropped, since there is no equivalent SQLite statement to fall back to.
    buf.push_str(&format!(
        "        // WARNING: SQLite cannot ALTER a column's type/nullable/unique constraint\n        // (sea-query panics on any `modify_column` there) — this change only applies on\n        // Postgres/MySQL; on SQLite the column keeps its current definition unchanged.\n        if manager.get_connection().get_database_backend() != sea_orm::DbBackend::Sqlite {{\n            manager\n                .alter_table(\n                    Table::alter()\n                        .table(Alias::new(\"{table}\"))\n                        .modify_column(ColumnDef::new(Alias::new(\"{col}\")).{ty}{null}{uniq})\n                        .to_owned(),\n                )\n                .await?;\n        }}\n\n",
        table = table,
        col = col,
        ty = col_type_method(column),
        null = null,
        uniq = uniq
    ));
}

fn push_add_column(buf: &mut String, table: &str, col: &ParsedColumn) {
    buf.push_str(&format!(
        "        manager\n            .alter_table(\n                Table::alter()\n                    .table(Alias::new(\"{table}\"))\n                    .add_column({coldef})\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        coldef = render_column_def(col),
    ));
}
