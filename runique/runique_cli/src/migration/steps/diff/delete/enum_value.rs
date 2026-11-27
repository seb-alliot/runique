//! A value taken out of an enum.

/// The values the snapshot has and the model has no more, the old side of a
/// rename aside, in the order the snapshot holds them.
pub(crate) fn enum_values(
    previous: &[String],
    current: &[String],
    renamed: &[(String, String)],
) -> Vec<String> {
    previous
        .iter()
        .filter(|value| !current.contains(value))
        .filter(|value| !renamed.iter().any(|(old, _)| old == *value))
        .cloned()
        .collect()
}
