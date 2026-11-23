//! What the tests of a batch need, here and in the crates that write through
//! one: a folder that goes with the test, and the journal of a batch cut short.
use crate::journal::{self, Journal, JournalFile};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// A directory under the system temp dir, removed on drop, even when the test fails.
pub struct TempDir(PathBuf);

impl Default for TempDir {
    fn default() -> Self {
        Self::new()
    }
}

impl TempDir {
    pub fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("rq_atomic_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    pub fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Every entry left under the directory, relative and sorted.
    pub fn entries(&self) -> Vec<String> {
        fn walk(root: &Path, dir: &Path, found: &mut Vec<String>) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                let relative = path.strip_prefix(root).unwrap().to_string_lossy();
                found.push(relative.replace('\\', "/"));
                if path.is_dir() {
                    walk(root, &path, found);
                }
            }
        }
        let mut found = Vec::new();
        walk(&self.0, &self.0, &mut found);
        found.sort();
        found
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Leaves in `dir` the journal of a batch cut short: each file by its name
/// and whether it was there before, then the folders the batch had created.
pub fn cut_journal(dir: &Path, placed: bool, files: &[(&str, bool)], dirs: &[&str]) {
    let journal = Journal {
        placed,
        files: files
            .iter()
            .map(|(name, existed)| JournalFile {
                path: PathBuf::from(name),
                existed: *existed,
                beside: false,
            })
            .collect(),
        dirs: dirs.iter().map(PathBuf::from).collect(),
    };
    journal::write(dir, &journal).unwrap();
}
