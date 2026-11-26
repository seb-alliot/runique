use super::*;

/// Adds the main-model migration files/modules to the plan. No snapshot:
/// `migration up` writes it once the migration is applied.
pub(crate) fn build_main_plan(
    plan: &mut Plan,
    all_changes: &[Changes],
    schemas: &[ParsedSchema],
    migrations_path: &str,
    timestamp: &str,
    settled: &[String],
) {
    let cycle_keys = cycle_keys(all_changes, schemas);
    let no_cycle = CycleKeys::default();
    for change in all_changes {
        let schema = schemas
            .iter()
            .find(|s| s.table_name == change.table_name)
            .unwrap();

        if change.table == TableChange::Created {
            let module_name = seaorm_create_module_name(timestamp, &change.table_name);
            let seaorm_path =
                seaorm_create_file_path(migrations_path, timestamp, &change.table_name);
            let keys = cycle_keys
                .get(change.table_name.as_str())
                .unwrap_or(&no_cycle);
            plan.files.push((
                seaorm_path,
                waiting_migrate(
                    generate_create_file_in_cycle(schema, keys),
                    &module_name,
                    settled,
                ),
            ));
            plan.lib_modules.push(module_name);
        } else {
            let module_name = seaorm_alter_module_name(timestamp, &change.table_name);
            let seaorm_path =
                seaorm_alter_file_path(migrations_path, timestamp, &change.table_name);
            // What `migration up` reads to take the former snapshot away.
            let renamed = change
                .renames
                .from
                .as_ref()
                .map_or_else(String::new, |old| format!("{RENAMED_TABLE}{old}\n"));
            plan.files.push((
                seaorm_path,
                waiting_migrate(
                    format!("{renamed}{}", generate_alter_file(change)),
                    &module_name,
                    settled,
                ),
            ));
            plan.lib_modules.push(module_name);
        }
    }
}
