//! An enum created.

/// Creates the enum `name` with `values`, through Runique: its type, unless
/// it is there already. Written for an engine that keeps an enum as an
/// object of its own; which one runs it is read when the migration runs.
pub(crate) fn enum_type(name: &str, values: &[String]) -> String {
    let name = name.to_lowercase();
    let values = values
        .iter()
        .map(|value| format!("{value:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "        runique_cli::migration::runtime::create_enum_type(manager, {name:?}, &[{values}]).await?;\n\n"
    )
}
