use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Writes the content to the temp file and flushes it to disk.
pub(super) fn write_temp(temp_path: &Path, content: &[u8], private_file: bool) -> io::Result<()> {
    remove(temp_path)?;
    if private_file {
        crate::utils::private_file::write(temp_path, content)?;
    } else {
        fs::write(temp_path, content)?;
    }
    fs::OpenOptions::new()
        .write(true)
        .open(temp_path)?
        .sync_all()
}

/// Makes renames done in `dir` survive a power loss.
/// Windows has no way to sync a directory, so this is a no-op there.
pub(super) fn sync_dir(dir: &Path) -> io::Result<()> {
    if cfg!(unix) {
        fs::File::open(dir)?.sync_all()
    } else {
        Ok(())
    }
}

pub(super) fn place(temp_path: &Path, path: &Path) -> io::Result<()> {
    match fs::hard_link(temp_path, path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Err(error),
        Err(_) => copy_new(temp_path, path),
    }
}

/// Fallback for filesystems without hard links: copies without overwriting,
/// and removes the partial copy on failure.
fn copy_new(temp_path: &Path, path: &Path) -> io::Result<()> {
    let mut source = fs::File::open(temp_path)?;
    let mut target = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let result = copy_content(&mut source, &mut target);
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result
}

fn copy_content(source: &mut fs::File, target: &mut fs::File) -> io::Result<()> {
    target.set_permissions(source.metadata()?.permissions())?;
    io::copy(source, target)?;
    target.sync_all()
}

/// Removes a file. A missing file is not an error, so a rollback can safely run twice.
pub(super) fn remove(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// Creates `dir` and its missing parents, recording each on it creates.
pub(crate) fn create_dirs(dir: &Path, created: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.as_os_str().is_empty() || dir.try_exists()? {
        return Ok(());
    }
    if let Some(parent) = dir.parent() {
        create_dirs(parent, created)?;
    }
    fs::create_dir(dir)?;
    created.push(dir.to_path_buf());
    Ok(())
}

/// Removes an empty directory. Already goe is not an error.
pub(super) fn remove_dir(dir: &Path) -> io::Result<()> {
    match fs::remove_dir(dir) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomic::tests::TempDir;

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[cfg(unix)]
    fn mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn place_links_the_content_and_leaves_the_source() {
        let dir = TempDir::new();
        fs::write(dir.join("source"), "content").unwrap();

        place(&dir.join("source"), &dir.join("target")).unwrap();

        assert_eq!(read(&dir.join("target")), "content");
        assert_eq!(dir.entries(), ["source", "target"]);
    }

    #[test]
    fn place_never_overwrites() {
        let dir = TempDir::new();
        fs::write(dir.join("source"), "content").unwrap();
        fs::write(dir.join("target"), "mine").unwrap();

        let outcome = place(&dir.join("source"), &dir.join("target"));

        assert_eq!(outcome.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(read(&dir.join("target")), "mine");
    }

    #[test]
    fn place_without_a_source_creates_nothing() {
        let dir = TempDir::new();

        let outcome = place(&dir.join("source"), &dir.join("target"));

        assert_eq!(outcome.unwrap_err().kind(), io::ErrorKind::NotFound);
        assert!(dir.entries().is_empty());
    }

    #[test]
    fn copy_new_copies_the_content_and_leaves_the_source() {
        let dir = TempDir::new();
        fs::write(dir.join("source"), "content").unwrap();

        copy_new(&dir.join("source"), &dir.join("target")).unwrap();

        assert_eq!(read(&dir.join("target")), "content");
        assert_eq!(read(&dir.join("source")), "content");
    }

    // The cleanup after a failed copy must not reach a file that was already there.
    #[test]
    fn copy_new_never_overwrites_nor_removes_an_existing_target() {
        let dir = TempDir::new();
        fs::write(dir.join("source"), "content").unwrap();
        fs::write(dir.join("target"), "mine").unwrap();

        let outcome = copy_new(&dir.join("source"), &dir.join("target"));

        assert_eq!(outcome.unwrap_err().kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(read(&dir.join("target")), "mine");
    }

    #[cfg(unix)]
    #[test]
    fn copy_new_carries_the_permissions_over() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new();
        fs::write(dir.join("source"), "secret").unwrap();
        fs::set_permissions(dir.join("source"), fs::Permissions::from_mode(0o600)).unwrap();

        copy_new(&dir.join("source"), &dir.join("target")).unwrap();

        assert_eq!(mode(&dir.join("target")), 0o600);
    }

    // A directory opens but cannot be read as a file: the copy fails once the
    // target has been created.
    #[cfg(unix)]
    #[test]
    fn copy_new_removes_its_partial_copy_when_the_copy_fails() {
        let dir = TempDir::new();
        fs::create_dir(dir.join("source")).unwrap();

        assert!(copy_new(&dir.join("source"), &dir.join("target")).is_err());
        assert_eq!(dir.entries(), ["source"]);
    }

    #[test]
    fn remove_deletes_a_file_and_accepts_a_missing_one() {
        let dir = TempDir::new();
        fs::write(dir.join("file"), "content").unwrap();

        remove(&dir.join("file")).unwrap();
        remove(&dir.join("file")).unwrap();

        assert!(dir.entries().is_empty());
    }

    #[test]
    fn remove_fails_on_a_directory_and_leaves_it() {
        let dir = TempDir::new();
        fs::create_dir(dir.join("folder")).unwrap();

        assert!(remove(&dir.join("folder")).is_err());
        assert_eq!(dir.entries(), ["folder"]);
    }

    #[test]
    fn create_dirs_records_only_what_it_creates() {
        let dir = TempDir::new();
        fs::create_dir(dir.join("kept")).unwrap();
        let mut created = Vec::new();

        create_dirs(&dir.join("kept/a/b"), &mut created).unwrap();
        create_dirs(&dir.join("kept/a/b"), &mut created).unwrap();

        assert_eq!(created, [dir.join("kept/a"), dir.join("kept/a/b")]);
        assert_eq!(dir.entries(), ["kept", "kept/a", "kept/a/b"]);
    }

    #[test]
    fn create_dirs_ignores_an_empty_path() {
        let mut created = Vec::new();

        create_dirs(Path::new(""), &mut created).unwrap();

        assert!(created.is_empty());
    }

    #[test]
    fn create_dirs_fails_through_a_file_and_records_nothing() {
        let dir = TempDir::new();
        fs::write(dir.join("blocker"), "a file").unwrap();
        let mut created = Vec::new();

        assert!(create_dirs(&dir.join("blocker/sub"), &mut created).is_err());
        assert!(created.is_empty());
        assert_eq!(dir.entries(), ["blocker"]);
    }

    #[test]
    fn remove_dir_only_removes_an_empty_directory() {
        let dir = TempDir::new();
        fs::create_dir(dir.join("empty")).unwrap();
        fs::create_dir(dir.join("full")).unwrap();
        fs::write(dir.join("full/file"), "mine").unwrap();

        remove_dir(&dir.join("empty")).unwrap();
        remove_dir(&dir.join("empty")).unwrap();
        assert!(remove_dir(&dir.join("full")).is_err());

        assert_eq!(dir.entries(), ["full", "full/file"]);
    }

    #[test]
    fn write_temp_writes_the_content() {
        let dir = TempDir::new();

        write_temp(&dir.join("temp"), b"content", false).unwrap();

        assert_eq!(read(&dir.join("temp")), "content");
    }

    #[cfg(unix)]
    #[test]
    fn write_temp_keeps_a_private_file_to_its_owner() {
        let dir = TempDir::new();

        write_temp(&dir.join("temp"), b"secret", true).unwrap();

        assert_eq!(mode(&dir.join("temp")), 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn sync_dir_fails_on_a_missing_directory() {
        let dir = TempDir::new();

        sync_dir(&dir.join("")).unwrap();
        assert!(sync_dir(&dir.join("missing")).is_err());
    }
}
