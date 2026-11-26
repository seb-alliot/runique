use super::*;

/// Makes `plan`, which holds the migrations just computed and nothing else,
/// take the place of the `waiting` ones. When both say the same thing, table
/// by table, nothing is rewritten: the plan is emptied and `false` returned.
/// Otherwise every waiting migration is dropped, except one the plan writes
/// again under the same name.
pub(crate) fn replace_waiting(plan: &mut Plan, waiting: &[(String, String)]) -> bool {
    // The same table and the same content: a creation and an alteration
    // never share one, so the content tells the kind too.
    fn same(waiting: &(String, String), module: &str, content: &str) -> bool {
        migration_target(module) == migration_target(&waiting.0) && content == waiting.1
    }
    let unchanged = waiting.len() == plan.lib_modules.len()
        && waiting.iter().all(|w| {
            plan.lib_modules
                .iter()
                .zip(&plan.files)
                .any(|(m, (_, c))| same(w, m, c))
        })
        && plan
            .lib_modules
            .iter()
            .zip(&plan.files)
            .all(|(m, (_, c))| waiting.iter().any(|w| same(w, m, c)));
    if unchanged {
        plan.files.clear();
        plan.lib_modules.clear();
        return false;
    }
    plan.dropped_modules.extend(
        waiting
            .iter()
            .map(|(module, _)| module)
            .filter(|module| !plan.lib_modules.contains(module))
            .cloned(),
    );
    true
}
