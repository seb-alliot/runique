pub(super) fn has_word(source: &str, word: &str) -> bool {
    source
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|found| found == word)
}
