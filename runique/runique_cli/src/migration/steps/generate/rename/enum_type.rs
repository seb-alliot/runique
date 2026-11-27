//! An enum renamed.

/// Gives the enum `from` the name `to`, through Runique: its type is renamed
/// in place, and every column that uses it follows.
pub(crate) fn enum_type(from: &str, to: &str) -> String {
    let (from, to) = (from.to_lowercase(), to.to_lowercase());
    format!(
        "        runique_cli::migration::runtime::rename_enum_type(manager, {from:?}, {to:?}).await?;\n\n"
    )
}
