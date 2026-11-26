use super::*;

/// A generated migration as it is written: tagged as waiting, and saying
/// which migration of `settled` it starts from for its table.
pub(crate) fn waiting_migrate(content: String, module: &str, settled: &[String]) -> String {
    let target = migration_target(module);
    let from = settled
        .iter()
        .rev()
        .find(|m| migration_target(m) == target)
        .map_or("none", String::as_str);
    format!("{WAIT_MIGRATION}\n{FROM_MIGRATION}{from}\n{content}")
}
