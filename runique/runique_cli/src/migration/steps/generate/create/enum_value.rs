//! A value added to an enum.

/// Adds `value` to the enum `name`, through Runique: to its type, unless it
/// has it already.
pub(crate) fn enum_value(name: &str, value: &str) -> String {
    let name = name.to_lowercase();
    format!(
        "        runique_cli::migration::runtime::add_enum_value(manager, {name:?}, {value:?}).await?;\n\n"
    )
}
