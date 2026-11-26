//! The order tables are dropped in.
use crate::migration::steps::changes::topological_sort_changes;

/// The order tables are dropped in: the reverse of the order they would be
/// created in, so a table always goes before the ones it references. Both
/// PostgreSQL and MySQL refuse to drop a table a foreign key still points
/// to, even when both tables are empty.
///
/// Returns the creation order (as [`topological_sort_changes`] gives it) of
/// `creations`, reversed. In a cycle of keys no order works: the keys left
/// pointing to a table not dropped yet are found by `cycle_keys` on the
/// creation order, and removed before that table.
pub fn drop_order(
    creations: Vec<crate::migration::types::Changes>,
) -> Vec<crate::migration::types::Changes> {
    let mut order = topological_sort_changes(creations);
    order.reverse();
    order
}
