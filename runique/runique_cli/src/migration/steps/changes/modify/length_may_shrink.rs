/// Whether going from `old` to `new` column length can drop data on some
/// engine. No length is unbounded on Postgres but `VARCHAR(255)` on MySQL, so
/// both directions out of "no length" are checked against each.
pub fn length_may_shrink(old: Option<u32>, new: Option<u32>) -> bool {
    match (old, new) {
        (Some(old), Some(new)) => new < old,
        (None, Some(_)) => true,
        (Some(old), None) => old > 255,
        (None, None) => false,
    }
}
