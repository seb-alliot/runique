use super::*;

/// The migrations of `lib.rs` no `migration up` applied yet (still tagged):
/// their module and their content.
pub(crate) fn waiting_migrations(migrations_path: &str) -> Result<Vec<(String, String)>> {
    let mut waiting = Vec::new();
    for module in lib_modules(migrations_path)? {
        if let Ok(content) = fs::read_to_string(format!("{migrations_path}/{module}.rs"))
            && has_marker(&content, WAIT_MIGRATION)
        {
            waiting.push((module, content));
        }
    }
    Ok(waiting)
}
