//! A Postgres enum type dropped.

/// Drops the Postgres enum type `name`, through Runique: the type is only
/// dropped once no column uses it any more (an enum shared by several tables
/// stays until the last one is gone), and never with a cascade. Every type
/// drop of the generator goes through here.
pub(crate) fn enum_type(name: &str) -> String {
    let name = name.to_lowercase();
    format!(
        "        runique_cli::migration::runtime::drop_enum_type(manager, {name:?}).await?;\n\n"
    )
}
