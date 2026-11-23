use crate::{
    disk,
    struct_atomic::{AtomicStep, ItemsPath},
};
use std::io;

/// The whole batch is `Done`: the backup and the temp file are no longer needed.
pub(crate) fn clean(item: &ItemsPath) -> io::Result<AtomicStep> {
    disk::remove(&item.backup_path)?;
    disk::remove(&item.temp_path)?;
    Ok(AtomicStep::Done)
}
