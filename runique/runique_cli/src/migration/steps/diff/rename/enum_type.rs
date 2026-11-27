//! An enum renamed.
use crate::migration::types::ParsedEnum;

/// (former name, name). An enum the models declare and the database doesn't
/// have is the one the database has under another name when that one is no
/// longer declared and a column that used it now uses the new one: the same
/// columns, another name for their enum. A former enum is taken once, by the
/// first enum that shares a column with it, in the order of `current`.
///
/// What the two hold is not compared: a renamed enum may change its values
/// in the same run, which is found afterwards, against the former ones.
pub(crate) fn enum_types(previous: &[ParsedEnum], current: &[ParsedEnum]) -> Vec<(String, String)> {
    let mut renamed: Vec<(String, String)> = Vec::new();
    let gone = previous
        .iter()
        .filter(|known| !current.iter().any(|declared| declared.name == known.name));
    let fresh: Vec<&ParsedEnum> = current
        .iter()
        .filter(|declared| !previous.iter().any(|known| known.name == declared.name))
        .collect();
    for former in gone {
        let heir = fresh.iter().find(|declared| {
            !renamed.iter().any(|(_, taken)| *taken == declared.name)
                && declared
                    .places
                    .iter()
                    .any(|place| former.places.contains(place))
        });
        if let Some(declared) = heir {
            renamed.push((former.name.clone(), declared.name.clone()));
        }
    }
    renamed
}
