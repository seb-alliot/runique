//! `TempDir` : un dossier sous le répertoire temporaire du système, supprimé
//! quand le garde est relâché — y compris quand le test échoue avant sa fin.

use std::path::{Path, PathBuf};

pub struct TempDir(PathBuf);

impl TempDir {
    /// `{temp}/{prefix}_{uuid}`, créé vide.
    pub fn new(prefix: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("{prefix}_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn join(&self, name: impl AsRef<Path>) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
