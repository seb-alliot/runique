/// Builds the FK constraint name. Length (MySQL hard-reject over 64 characters) is
/// validated upfront, over the whole change set, before any file generation runs — see
/// `check_identifier_lengths` in `migration/steps/changes/report.rs` — so this stays a pure formatter.
/// The create and drop statements for the same FK must always agree on the exact name, so
/// this is the single place that builds it.
pub(crate) fn fk_constraint_name(table: &str, from: &str, to_table: &str) -> String {
    format!("{table}_{from}_{to_table}_fkey")
}
