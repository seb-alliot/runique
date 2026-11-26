/// Whether going from `old` to `new` decimal precision can drop data. No
/// precision is unbounded on Postgres and `DECIMAL(10, 0)` on MySQL, so a
/// first precision always may: fewer digits before the point, or after it.
pub fn decimal_may_shrink(old: Option<(u32, u32)>, new: Option<(u32, u32)>) -> bool {
    match (old, new) {
        (Some((old_p, old_s)), Some((new_p, new_s))) => {
            new_s < old_s || new_p.saturating_sub(new_s) < old_p.saturating_sub(old_s)
        }
        (None, Some(_)) => true,
        (_, None) => false,
    }
}
