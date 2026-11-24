//! The marker lines Runique writes in the files it generates: found and
//! removed as whole lines, wherever they are in the file.

/// Whether `content` holds `marker` on a line of its own, wherever it is.
pub fn has_marker(content: &str, marker: &str) -> bool {
    content.lines().any(|line| line.trim() == marker)
}

/// `content` without the lines that are `marker`; the rest byte for byte.
pub fn without_marker(content: &str, marker: &str) -> String {
    content
        .split_inclusive('\n')
        .filter(|line| line.trim() != marker)
        .collect()
}
