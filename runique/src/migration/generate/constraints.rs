//! Foreign keys and indexes: their names, and their create and drop statements.
use super::indent;
use crate::migration::types::{ParsedFk, ParsedIndex, ParsedSchema};

/// Builds the FK constraint name. Length (MariaDB/MySQL hard-reject over 64 characters) is
/// validated upfront, over the whole change set, before any file generation runs — see
/// `check_identifier_lengths` in `migration/cli_migration/report.rs` — so this stays a pure formatter.
/// The create and drop statements for the same FK must always agree on the exact name, so
/// this is the single place that builds it.
pub(super) fn fk_constraint_name(table: &str, from: &str, to_table: &str) -> String {
    format!("{table}_{from}_{to_table}_fkey")
}

/// `ForeignKey::create()…` for the key `fk` of `table`, each line indented by `indent`.
pub(super) fn render_fk_create(table: &str, fk: &ParsedFk, indent: &str) -> String {
    format!(
        "{indent}ForeignKey::create()\n{indent}    .name(\"{name}\")\n{indent}    .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n{indent}    .to(Alias::new(\"{to_table}\"), Alias::new(\"{to_col}\"))\n{indent}    .on_delete(ForeignKeyAction::{on_delete})\n{indent}    .on_update(ForeignKeyAction::{on_update})",
        name = fk_constraint_name(table, &fk.from_column, &fk.to_table),
        from = fk.from_column,
        to_table = fk.to_table,
        to_col = fk.to_column,
        on_delete = fk.on_delete,
        on_update = fk.on_update,
    )
}

pub(super) fn build_fk_create_stmts(schema: &ParsedSchema) -> String {
    let mut out = String::new();
    for fk in &schema.foreign_keys {
        out.push_str(&format!(
            "        manager\n            .create_foreign_key(\n                ForeignKey::create()\n                    .name(\"{fk_name}\")\n                    .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n                    .to(Alias::new(\"{to_table}\"), Alias::new(\"{to_col}\"))\n                    .on_delete(ForeignKeyAction::{on_delete})\n                    .on_update(ForeignKeyAction::{on_update})\n                    .to_owned(),\n            )\n            .await?;\n\n",
            fk_name = fk_constraint_name(&schema.table_name, &fk.from_column, &fk.to_table),
            table = schema.table_name,
            from = fk.from_column,
            to_table = fk.to_table,
            to_col = fk.to_column,
            on_delete = fk.on_delete,
            on_update = fk.on_update
        ));
    }
    out
}

/// `guard_fk_indexes`: wrap a foreign key's own index in the MySQL check of
/// [`unless_mysql_fk`] — for the files that run, never for snapshots, which
/// `parse/seaorm.rs` reads back as plain statements.
pub(super) fn build_index_create_stmts(schema: &ParsedSchema, guard_fk_indexes: bool) -> String {
    let mut out = String::new();
    for idx in &schema.indexes {
        let stmt =
            render_create_index_stmt(&schema.table_name, &idx.name, &idx.columns, idx.unique);
        match fk_index_column(&schema.table_name, idx).filter(|_| guard_fk_indexes) {
            Some(col) => out.push_str(&unless_mysql_fk(&schema.table_name, col, &stmt)),
            None => out.push_str(&stmt),
        }
        out.push('\n');
    }
    out
}

/// The column of a foreign key's own index (`idx_{table}_{column}`, one column,
/// not unique) — what `to_schema` adds for every `belongs_to`.
fn fk_index_column<'a>(table: &str, idx: &'a ParsedIndex) -> Option<&'a str> {
    match idx.columns.as_slice() {
        [col] if !idx.unique && idx.name == format!("idx_{table}_{col}") => Some(col),
        _ => None,
    }
}

/// Runs `stmt` unless the database is MySQL/MariaDB and `column` holds a
/// foreign key there. InnoDB already indexes every foreign key column: an
/// explicit index would replace that one and then couldn't be dropped while
/// the key exists (error 1553, which broke `migrate down`). Checked at run time
/// so one file works on every engine.
fn unless_mysql_fk(table: &str, column: &str, stmt: &str) -> String {
    format!(
        r#"        {{
            let backs_fk = manager.get_connection().get_database_backend() == sea_orm::DbBackend::MySql
                && manager
                    .get_connection()
                    .query_one(
                        &Query::select()
                            .expr(Expr::val(1))
                            .from((Alias::new("information_schema"), Alias::new("KEY_COLUMN_USAGE")))
                            .and_where(Expr::col(Alias::new("TABLE_SCHEMA")).eq(Expr::cust("DATABASE()")))
                            .and_where(Expr::col(Alias::new("TABLE_NAME")).eq({table:?}))
                            .and_where(Expr::col(Alias::new("COLUMN_NAME")).eq({column:?}))
                            .and_where(Expr::col(Alias::new("REFERENCED_TABLE_NAME")).is_not_null())
                            .to_owned(),
                    )
                    .await?
                    .is_some();
            if !backs_fk {{
{body}            }}
        }}
"#,
        body = indent(stmt, "        "),
    )
}

pub(super) fn build_fk_drop_stmts(schema: &ParsedSchema) -> String {
    let mut out = String::new();
    for fk in &schema.foreign_keys {
        out.push_str(&format!(
            "        manager\n            .drop_foreign_key(\n                ForeignKey::drop()\n                    .table(Alias::new(\"{table}\"))\n                    .name(\"{fk_name}\")\n                    .to_owned(),\n            )\n            .await?;\n\n",
            fk_name = fk_constraint_name(&schema.table_name, &fk.from_column, &fk.to_table),
            table = schema.table_name,
        ));
    }
    out
}

pub(super) fn build_index_drop_stmts(schema: &ParsedSchema) -> String {
    let mut out = String::new();
    for idx in &schema.indexes {
        out.push_str(&format!(
            "        manager\n            .drop_index(Index::drop().name(\"{idx}\").table(Alias::new(\"{table}\")).to_owned())\n            .await?;\n\n",
            idx = idx.name,
            table = schema.table_name
        ));
    }
    out
}

pub(super) fn push_drop_index(buf: &mut String, table: &str, idx: &ParsedIndex) {
    let stmt = format!(
        "        manager\n            .drop_index(Index::drop().name(\"{name}\").table(Alias::new(\"{table}\")).to_owned())\n            .await?;\n",
        name = idx.name,
        table = table
    );
    match fk_index_column(table, idx) {
        Some(col) => buf.push_str(&unless_mysql_fk(table, col, &stmt)),
        None => buf.push_str(&stmt),
    }
    buf.push('\n');
}

pub(super) fn push_drop_fk(buf: &mut String, table: &str, from_col: &str, to_table: &str) {
    buf.push_str(&format!(
        "        manager\n            .drop_foreign_key(\n                ForeignKey::drop()\n                    .table(Alias::new(\"{table}\"))\n                    .name(\"{fk_name}\")\n                    .to_owned(),\n            )\n            .await?;\n\n",
        fk_name = fk_constraint_name(table, from_col, to_table),
        table = table,
    ));
}

pub(super) fn push_create_fk(
    buf: &mut String,
    table: &str,
    from_col: &str,
    to_table: &str,
    to_col: &str,
    on_delete: &str,
    on_update: &str,
) {
    buf.push_str(&format!(
        "        manager\n            .create_foreign_key(\n                ForeignKey::create()\n                    .name(\"{fk_name}\")\n                    .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n                    .to(Alias::new(\"{to_table}\"), Alias::new(\"{to_col}\"))\n                    .on_delete(ForeignKeyAction::{on_delete})\n                    .on_update(ForeignKeyAction::{on_update})\n                    .to_owned(),\n            )\n            .await?;\n\n",
        fk_name = fk_constraint_name(table, from_col, to_table),
        table = table,
        from = from_col,
        to_table = to_table,
        to_col = to_col,
        on_delete = on_delete,
        on_update = on_update
    ));
}

pub(super) fn push_create_index(buf: &mut String, table: &str, idx: &ParsedIndex) {
    let stmt = render_create_index_stmt(table, &idx.name, &idx.columns, idx.unique);
    match fk_index_column(table, idx) {
        Some(col) => buf.push_str(&unless_mysql_fk(table, col, &stmt)),
        None => buf.push_str(&stmt),
    }
    buf.push('\n');
}

fn render_create_index_stmt(
    table: &str,
    idx_name: &str,
    columns: &[String],
    unique: bool,
) -> String {
    let mut cols_chain = String::new();
    for c in columns {
        cols_chain.push_str(&format!(
            "                    .col(Alias::new(\"{c}\"))\n",
            c = c
        ));
    }

    let uniq_line = if unique {
        "                    .unique()\n"
    } else {
        ""
    };

    format!(
        "        manager\n            .create_index(\n                Index::create()\n                    .name(\"{idx}\")\n                    .table(Alias::new(\"{table}\"))\n{cols}{uniq}                    .to_owned(),\n            )\n            .await?;\n",
        idx = idx_name,
        table = table,
        cols = cols_chain,
        uniq = uniq_line
    )
}
