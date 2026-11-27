//! A value added to an enum.

/// The values the model has and the snapshot has not, the new side of a
/// rename aside, in the order the model declares them.
pub(crate) fn enum_values(
    previous: &[String],
    current: &[String],
    renamed: &[(String, String)],
) -> Vec<String> {
    current
        .iter()
        .filter(|value| !previous.contains(value))
        .filter(|value| !renamed.iter().any(|(_, new)| new == *value))
        .cloned()
        .collect()
}
