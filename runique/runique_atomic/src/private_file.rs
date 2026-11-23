//! Files holding a secret (`.env`, a TLS private key): readable by their owner only.

use std::io::Write;
use std::path::Path;

/// Writes `content` to `path`, readable and writable by its owner only on Unix
/// (`0600`), whether the file is new or already there.
pub fn write(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    // `mode` only applies to a file it creates: one already there keeps its own.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
    }
    file.write_all(content)?;
    file.flush()
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    fn mode(path: &std::path::Path) -> u32 {
        std::fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn a_secret_file_is_its_owners_only_new_or_rewritten() {
        let dir = std::env::temp_dir().join(format!("rq_private_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("key.pem");
        super::write(&path, b"secret").unwrap();
        assert_eq!(mode(&path), 0o600);
        assert_eq!(std::fs::read(&path).unwrap(), b"secret");

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        super::write(&path, b"renewed").unwrap();
        assert_eq!(mode(&path), 0o600);
        assert_eq!(std::fs::read(&path).unwrap(), b"renewed");

        let other = dir.join("cert.pem");
        std::fs::write(&other, b"public").unwrap();
        assert_ne!(mode(&other), 0o600, "an ordinary write is left as it was");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
