use super::*;

/// Runs `read`, which reads the models, with `Pk` as the project has it.
/// `pk` is only asked when a file of `entities_path` writes `Pk`: a project
/// that spells its key types out never pays for the question.
pub(crate) fn with_project_pk<T>(
    entities_path: &str,
    pk: impl FnOnce() -> Result<PkType>,
    read: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if !alias_used(Path::new(entities_path)) {
        return read();
    }
    runique_dsl::with_project_pk(pk()?, read)
}
