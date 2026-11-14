use std::{fs, io, path::Path};

/// Writes the content to the temp file and flushes it to disk.
pub(super) fn write_temp(temp_path: &Path, content: &[u8], private_file: bool) -> io::Result<()> {
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
