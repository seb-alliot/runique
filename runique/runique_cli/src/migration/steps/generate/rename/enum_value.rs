//! A value of an enum renamed.
use crate::migration::types::EnumPlace;

/// Renames the value `from` of the enum `name`, through Runique, once for
/// the enum: in its type where it has one, in the rows of each of `places`
/// where its columns are text.
pub(crate) fn enum_value(name: &str, from: &str, to: &str, places: &[EnumPlace]) -> String {
    let name = name.to_lowercase();
    let places = places
        .iter()
        .map(|place| format!("({:?}, {:?})", place.table, place.column))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "        runique_cli::migration::runtime::rename_enum_value(manager, {name:?}, {from:?}, {to:?}, &[{places}]).await?;\n\n"
    )
}
