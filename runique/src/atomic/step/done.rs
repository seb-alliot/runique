use crate::atomic::{
    action::{finish, rollback},
    struct_atomic::{ActionAtomic, AtomicStep, ItemsPath},
};
use std::io;

/// The file is at its final location. Only `Finish` still removes anything.
pub(in crate::atomic) fn run(action: &ActionAtomic, item: &ItemsPath) -> io::Result<AtomicStep> {
    match action {
        ActionAtomic::Create => Ok(AtomicStep::Done),
        ActionAtomic::Modify => Ok(AtomicStep::Done),
        ActionAtomic::Rollback => undo(item),
        ActionAtomic::Finish => finish::clean(item),
    }
}

/// A backup means there was an original: put it back.
fn undo(item: &ItemsPath) -> io::Result<AtomicStep> {
    if item.backup_path.try_exists()? {
        rollback::restore(item)
    } else {
        rollback::remove(item)
    }
}
