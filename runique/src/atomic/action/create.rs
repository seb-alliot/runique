use crate::atomic::{
    disk,
    struct_atomic::{AtomicStep, ItemsPath, SourceItems},
};
use std::io;

/// Writes the temp file. A creation always starts from in-memory content.
pub(in crate::atomic) fn prepare(item: &ItemsPath) -> io::Result<AtomicStep> {
    match &item.source {
        SourceItems::Bytes(content) => {
            disk::write_temp(&item.temp_path, content, item.private_file)?;
            Ok(AtomicStep::Prepared)
        }
        SourceItems::File(_) => Err(io::ErrorKind::InvalidInput.into()),
    }
}

/// Puts the file at its final location. Fails if `path` exists: a creation never overwrites.
pub(in crate::atomic) fn place(item: &ItemsPath) -> io::Result<AtomicStep> {
    disk::place(&item.temp_path, &item.path)?;
    Ok(AtomicStep::Placed)
}
