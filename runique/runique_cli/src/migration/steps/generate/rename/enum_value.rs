//! A value of an enum renamed.

// Enum value renames — both branches (Postgres native `RENAME VALUE` vs.
// VARCHAR-backed `UPDATE`) are always emitted; which one runs is decided at
// migration-run time via the real backend, not baked in at generation time
// (a migration file is fixed forever once written — see the enum CREATE/DROP
// functions (`create::enum_type`) for the same reasoning).
// `ALTER TYPE ... RENAME VALUE`/`ADD VALUE` go through
// `sea_query::extension::postgres::Type`, like the creation and the drop of
// the type; the type name is lowercased the same way (the builder quotes it).
pub(crate) fn enum_value(
    table: &str,
    column: &str,
    enum_name: &str,
    from: &str,
    to: &str,
) -> String {
    format!(
        "        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {{\n            manager\n                .alter_type(\n                    sea_query::extension::postgres::Type::alter()\n                        .name(Alias::new(\"{enum_name_lc}\"))\n                        .rename_value(Alias::new({from:?}), Alias::new({to:?}))\n                        .to_owned(),\n                )\n                .await?;\n        }} else {{\n            manager\n                .get_connection()\n                .execute(\n                    &Query::update()\n                        .table(Alias::new(\"{table}\"))\n                        .value(Alias::new(\"{col}\"), {to:?})\n                        .and_where(Expr::col(Alias::new(\"{col}\")).eq({from:?}))\n                        .to_owned(),\n                )\n                .await?;\n        }}\n\n",
        enum_name_lc = enum_name.to_lowercase(),
        from = from,
        to = to,
        table = table,
        col = column,
    )
}
