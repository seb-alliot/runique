pub(super) mod create;
pub(super) mod finish;
pub(super) mod modify;
pub(super) mod rollback;

use crate::atomic::{
    disk,
    struct_atomic::{AtomicStep, ItemsPath, SourceItems},
};
use std::io;

/// Writes the temp file, from memory or from a file already on disk. The same
/// for a creation and for a modification: they only differ once `path` is reached.
pub(super) fn prepare(item: &ItemsPath) -> io::Result<AtomicStep> {
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
