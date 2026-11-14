use crate::atomic::{
    disk,
    struct_atomic::{AtomicStep, ItemsPath},
};
use std::{fs, io};

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
