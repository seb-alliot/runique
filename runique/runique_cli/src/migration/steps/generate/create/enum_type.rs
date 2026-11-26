//! A Postgres enum type created.

// What creates or drops an enum type writes a `CREATE TYPE`/`DROP TYPE` — Postgres-only syntax —
// unconditionally into the generated migration, wrapped in a runtime
// `manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres`
// check rather than gated by `db_kind` at generation time. A migration file is
// fixed forever once written; deciding "include this or not" from whatever engine
// `makemigrations` happened to target freezes the file to that ONE engine
// (confirmed in practice: a Postgres-generated `CREATE TYPE ... AS ENUM` run against
// MySQL is a straight syntax error). Checking the real backend at migration-RUN
// time instead makes the same generated file portable across all three.

/// Creates the Postgres enum type `name`, unless it exists already — Postgres
/// has no `CREATE TYPE IF NOT EXISTS`, so the catalogue is looked up first (an
/// enum shared by several tables is created by the first migration only).
///
/// The name is lowercased: the builder quotes type names, and every other
/// reference to the type (`ColumnType::Enum`, `Type::alter()` below) uses the
/// lowercase form.
pub(crate) fn enum_type(name: &str, values: &[String]) -> String {
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
