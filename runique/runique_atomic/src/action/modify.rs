use crate::{
    disk,
    struct_atomic::{AtomicStep, ItemsPath, SourceItems},
};
use std::{fs, io};

/// Puts the new content at `path`. If a file was there, it is kept at `backup_path`.
pub(crate) fn replace(item: &ItemsPath) -> io::Result<AtomicStep> {
    let removal = matches!(item.source, SourceItems::Remove);
    match disk::place(&item.path, &item.backup_path) {
        Ok(()) => {
            fs::remove_file(&item.path)?;
            if !removal {
                disk::place(&item.temp_path, &item.path)?;
            }
            Ok(AtomicStep::Replaced)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound && !removal => {
            disk::place(&item.temp_path, &item.path)?;
            Ok(AtomicStep::Placed)
        }
        Err(error) => Err(error),
    }
}
