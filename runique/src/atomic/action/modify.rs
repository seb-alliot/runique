use crate::atomic::{
    disk,
    struct_atomic::{AtomicStep, ItemsPath, SourceItems},
};
use std::{fs, io};

/// Writes the temp file, from memory or from a file already on disk.
pub(in crate::atomic) fn prepare(item: &ItemsPath) -> io::Result<AtomicStep> {
    match &item.source {
        SourceItems::Bytes(content) => {
            disk::write_temp(&item.temp_path, content, item.private_file)?
        }
        SourceItems::File(source) => {
            disk::remove(&item.temp_path)?;
            disk::place(source, &item.temp_path)?
        }
    }
    Ok(AtomicStep::Prepared)
}

/// Puts the new content at `path`. If a file was there, it is kept at `backup_path`.
pub(in crate::atomic) fn replace(item: &ItemsPath) -> io::Result<AtomicStep> {
    match disk::place(&item.path, &item.backup_path) {
        Ok(()) => {
            fs::remove_file(&item.path)?;
            disk::place(&item.temp_path, &item.path)?;
            Ok(AtomicStep::Replaced)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            disk::place(&item.temp_path, &item.path)?;
            Ok(AtomicStep::Placed)
        }
        Err(error) => Err(error),
    }
}
