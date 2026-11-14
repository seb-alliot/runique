use crate::atomic::{
    disk,
    struct_atomic::{AtomicStep, ItemsPath},
};
use std::io;

/// Puts the file at its final location. Fails if `path` exists: a creation never overwrites.
pub(in crate::atomic) fn place(item: &ItemsPath) -> io::Result<AtomicStep> {
    disk::place(&item.temp_path, &item.path)?;
    Ok(AtomicStep::Placed)
}
