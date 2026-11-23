use crate::{
    disk,
    struct_atomic::{AtomicStep, ItemsPath},
};
use std::io;

/// Removes the temp file: `path` was never touched.
pub(crate) fn remove_temp(item: &ItemsPath) -> io::Result<AtomicStep> {
    disk::remove(&item.temp_path)?;
    Ok(AtomicStep::Waiting)
}

/// Removes the placed file: nothing was there before.
pub(crate) fn remove(item: &ItemsPath) -> io::Result<AtomicStep> {
    disk::remove(&item.path)?;
    remove_temp(item)
}

/// Puts the original back from `backup_path`.
pub(crate) fn restore(item: &ItemsPath) -> io::Result<AtomicStep> {
    disk::remove(&item.path)?;
    disk::place(&item.backup_path, &item.path)?;
    disk::remove(&item.temp_path)?;
    disk::remove(&item.backup_path)?;
    Ok(AtomicStep::Waiting)
}
