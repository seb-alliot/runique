use super::*;

/// Whether a `.rs` file of the folder holds the word `Pk`. One in a comment
/// counts too: it costs a question to cargo, never a wrong type. A folder
/// that can't be read answers no: reading the models reports it.
pub(super) fn alias_used(entities: &Path) -> bool {
    let Ok(files) = crate::migration::steps::parser::files::rust_files::rust_files(entities) else {
        return false;
    };
    files
        .into_iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .any(|source| has_word(&source, "Pk"))
}
