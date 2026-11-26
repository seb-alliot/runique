use super::*;

/// Adds the extend!{} migration files/modules to the plan. No snapshot:
/// `migration up` writes it once the migration is applied.
pub(crate) fn build_extend_plan(
    plan: &mut Plan,
    planned: &[(ParsedSchema, Changes)],
    migrations_path: &str,
    timestamp: &str,
    settled: &[String],
) {
    if planned.is_empty() {
        return;
    }

    for (ext_schema, changes) in planned {
        let module_name = seaorm_extend_module_name(timestamp, &ext_schema.table_name);
        let seaorm_path =
            seaorm_extend_file_path(migrations_path, timestamp, &ext_schema.table_name);
        plan.files.push((
            seaorm_path,
            waiting_migrate(generate_alter_file(changes), &module_name, settled),
        ));
        plan.lib_modules.push(module_name);
    }
}
