//! A column redefined: its length, its precision, whether it is required or unique.
use crate::migration::{steps::generate::columns::col_type_method, types::ParsedColumn};

pub(crate) fn column(table: &str, column: &ParsedColumn) -> String {
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
    format!(
        "        // WARNING: SQLite cannot ALTER a column's type/nullable/unique constraint\n        // (sea-query panics on any `modify_column` there) — this change only applies on\n        // Postgres/MySQL; on SQLite the column keeps its current definition unchanged.\n        if manager.get_connection().get_database_backend() != sea_orm::DbBackend::Sqlite {{\n            manager\n                .alter_table(\n                    Table::alter()\n                        .table(Alias::new(\"{table}\"))\n                        .modify_column(ColumnDef::new(Alias::new(\"{col}\")).{ty}{null}{uniq})\n                        .to_owned(),\n                )\n                .await?;\n        }}\n\n",
        table = table,
        col = col,
        ty = col_type_method(column),
        null = null,
        uniq = uniq
    )
}
