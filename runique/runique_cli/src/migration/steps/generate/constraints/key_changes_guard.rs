/// What a migration that changes the keys of the existing table `table`
/// runs first, in `up` and in `down`: the engine is asked whether it can,
/// before any statement of the migration.
pub(crate) fn key_changes_guard(table: &str) -> String {
    format!(
        "        runique_cli::migration::runtime::key_changes_supported(manager, \"{table}\")?;\n\n"
    )
}
