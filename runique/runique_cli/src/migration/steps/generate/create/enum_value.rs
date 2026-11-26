//! A value added to an enum.

// Enum value additions — Postgres native enum only; VARCHAR-backed
// enums (non-PG) accept any string, so no DDL is needed there. Guarded at
// runtime rather than skipped at generation time, for the reason given
// in `create/enum_type.rs`.
pub(crate) fn enum_value(enum_name: &str, value: &str) -> String {
    format!(
        "        if manager.get_connection().get_database_backend() == sea_orm::DbBackend::Postgres {{\n            manager\n                .alter_type(\n                    sea_query::extension::postgres::Type::alter()\n                        .name(Alias::new(\"{enum_name_lc}\"))\n                        .add_value(Alias::new({val:?}))\n                        .if_not_exists()\n                        .to_owned(),\n                )\n                .await?;\n        }}\n\n",
        enum_name_lc = enum_name.to_lowercase(),
        val = value,
    )
}
