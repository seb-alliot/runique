use crate::atomic::{
    action::{create, modify},
    struct_atomic::{ActionAtomic, AtomicStep, ItemsPath},
};
use std::io;

/// Nothing has been done for this file yet.
pub(in crate::atomic) fn run(action: &ActionAtomic, item: &ItemsPath) -> io::Result<AtomicStep> {
    match action {
        ActionAtomic::Create => {
            check(item)?;
            create::prepare(item)
        }
        ActionAtomic::Modify => {
            check(item)?;
            modify::prepare(item)
        }
        ActionAtomic::Rollback => Ok(AtomicStep::Waiting),
        ActionAtomic::Finish => Err(io::ErrorKind::InvalidInput.into()),
    }
}

/// A leftover backup belongs to an earlier batch: starting over it could
/// put stale content back in place of the user's file.
fn check(item: &ItemsPath) -> io::Result<()> {
    if item.backup_path.try_exists()? {
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    Ok(())
}
