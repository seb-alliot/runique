//! The files of a folder.
use crate::utils::trad::tf;
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// The `.rs` files of `dir`, sorted by name. A folder named like one
/// (`archive.rs/`) is refused: skipped in silence, it would hide a mistake
/// the developer can fix in a second.
pub(crate) fn rust_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let entries =
        fs::read_dir(dir).with_context(|| format!("Cannot read directory: {}", dir.display()))?;
    let mut files: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let path = entry?.path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        if path.is_dir() {
            anyhow::bail!(
                "{}",
                tf(
                    runique_i18n::keys::makemigrations::error::FOLDER_NAMED_RS,
                    &[path.display()]
                )
            );
        }
        files.push(path);
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomic::tests::TempDir;

    fn names(files: &[PathBuf]) -> Vec<&str> {
        files
            .iter()
            .map(|path| path.file_name().unwrap().to_str().unwrap())
            .collect()
    }

    // The `.rs` files and nothing else, by name whatever the order they were
    // written in; a folder that isn't named like one is left out.
    #[test]
    fn the_rust_files_of_a_folder_come_sorted_and_alone() {
        let dir = TempDir::new();
        for name in [
            "zebra.rs",
            "alpha.rs",
            "mod.rs",
            "notes.txt",
            "bare",
            "rs",
            ".rs",
        ] {
            fs::write(dir.join(name), "").unwrap();
        }
        fs::create_dir(dir.join("archive")).unwrap();
        fs::write(dir.join("archive/inner.rs"), "").unwrap();

        let files = rust_files(dir.path()).unwrap();

        assert_eq!(names(&files), ["alpha.rs", "mod.rs", "zebra.rs"]);
        assert!(files.iter().all(|path| path.starts_with(dir.path())));
    }

    #[test]
    fn a_folder_without_a_rust_file_gives_none() {
        let dir = TempDir::new();
        assert!(rust_files(dir.path()).unwrap().is_empty());
        fs::write(dir.join("notes.txt"), "").unwrap();
        assert!(rust_files(dir.path()).unwrap().is_empty());
    }

    // A folder named like a `.rs` file stops the reading, by its name, even
    // when the real files around it are fine.
    #[test]
    fn a_folder_named_like_a_rust_file_is_refused_by_its_name() {
        let dir = TempDir::new();
        fs::write(dir.join("menu.rs"), "").unwrap();
        fs::create_dir(dir.join("archive.rs")).unwrap();

        let error = rust_files(dir.path()).unwrap_err().to_string();

        assert!(error.contains("archive.rs"), "{error}");
        assert!(!error.contains("menu.rs"), "{error}");
    }

    #[test]
    fn a_folder_that_cannot_be_read_is_an_error_naming_it() {
        let dir = TempDir::new();
        let missing = dir.join("nowhere");

        let error = rust_files(&missing).unwrap_err().to_string();

        assert!(error.contains("nowhere"), "{error}");
    }
}
