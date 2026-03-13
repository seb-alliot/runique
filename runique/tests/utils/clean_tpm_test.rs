use std::fs;
use std::path::PathBuf;

/// Temp dir auto-deleted on drop (even on panic).
pub struct TestTempDir(pub PathBuf);

impl TestTempDir {
    pub fn new(prefix: &str, suffix: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("{}_{}", prefix, suffix));
        fs::create_dir_all(&dir).ok();
        TestTempDir(dir)
    }

    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap()
    }
}

impl Drop for TestTempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).ok();
    }
}

impl std::ops::Deref for TestTempDir {
    type Target = PathBuf;
    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

impl AsRef<std::path::Path> for TestTempDir {
    fn as_ref(&self) -> &std::path::Path {
        &self.0
    }
}

impl std::fmt::Display for TestTempDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}
