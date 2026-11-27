use super::*;

/// Adds one migration per table to drop, after everything else of the plan:
/// the tables that reference go before the ones they reference, and in a
/// cycle of keys the key still pointing to a table is removed before it.
/// The snapshot stays: `migration up` removes it once the drop is applied.
pub(crate) fn build_drop_plan(
    plan: &mut Plan,
    dropped: &[ParsedSchema],
    migrations_path: &str,
    timestamp: &str,
) {
    // Reversed creation order; the keys of a cycle are worked out on the
    // creation order, which is the order the `down`s run in.
    let creation: Vec<Changes> =
        topological_sort_changes(dropped.iter().map(creation_of).collect());
    let cycle_keys = cycle_keys(&creation, dropped);
    let no_cycle = CycleKeys::default();
    for change in drop_order(creation.clone()) {
        let Some(schema) = dropped.iter().find(|s| s.table_name == change.table_name) else {
            continue;
        };
        let keys = cycle_keys
            .get(schema.table_name.as_str())
            .unwrap_or(&no_cycle);
        let module_name = seaorm_drop_module_name(timestamp, &schema.table_name);
        plan.files.push((
            seaorm_drop_file_path(migrations_path, timestamp, &schema.table_name),
            generate_drop_file(schema, keys),
        ));
        plan.lib_modules.push(module_name);
    }
}
