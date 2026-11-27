//! A value of an enum renamed.

/// (old value, new value). A rename is ONE logical operation: its values are
/// left out of what is added and dropped, so the generator can rename the
/// value instead of adding one, dropping the other and losing the rows.
/// Found by position: same index, value changed, old value gone, new value
/// fresh.
pub(crate) fn enum_values(previous: &[String], current: &[String]) -> Vec<(String, String)> {
    current
        .iter()
        .enumerate()
        .filter_map(|(index, new)| {
            let old = previous.get(index)?;
            (old != new && !current.contains(old) && !previous.contains(new))
                .then(|| (old.clone(), new.clone()))
        })
        .collect()
}
