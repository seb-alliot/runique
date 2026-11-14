use crate::atomic::{
    action::{create, modify, rollback},
    struct_atomic::{ActionAtomic, AtomicStep, ItemsPath},
};
use std::io;

/// The temp file is written, `path` does not hold the new content yet.
pub(in crate::atomic) fn run(action: &ActionAtomic, item: &ItemsPath) -> io::Result<AtomicStep> {
    match action {
        ActionAtomic::Create => create::place(item),
        ActionAtomic::Modify => modify::replace(item),
        ActionAtomic::Rollback => undo(item),
        ActionAtomic::Finish => Err(io::ErrorKind::InvalidInput.into()),
    }
}

/// `replace` may have stopped right after the backup: if one exists, the original goes back.
fn undo(item: &ItemsPath) -> io::Result<AtomicStep> {
    if item.backup_path.try_exists()? {
        rollback::restore(item)
    } else {
        rollback::remove_temp(item)
    }
}
