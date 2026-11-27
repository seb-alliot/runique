use super::*;

/// Adds the migration files/modules of `changes` to the plan: one file per
/// enum, written for `engine`. No snapshot: `migration up` writes it once
/// the migration is applied.
pub(crate) fn build_enum_plan<'a>(
    plan: &mut Plan,
    changes: impl IntoIterator<Item = &'a EnumChanges>,
    migrations_path: &str,
    timestamp: &str,
    engine: &DatabaseEngine,
) {
    for change in changes {
        let module_name = match change.change {
            EnumChange::Created => seaorm_enum_create_module_name(timestamp, &change.name),
            EnumChange::Renamed => seaorm_enum_rename_module_name(timestamp, &change.name),
            EnumChange::Altered => seaorm_enum_alter_module_name(timestamp, &change.name),
        };
        plan.files.push((
            seaorm_enum_file_path(migrations_path, &module_name),
            generate_enum_file(change, engine),
        ));
        plan.lib_modules.push(module_name);
    }
}
