use crate::atomic::{
    action::prepare,
    struct_atomic::{ActionAtomic, AtomicStep, ItemsPath},
};
use std::io;

/// Nothing has been done for this file yet.
pub(in crate::atomic) fn run(action: &ActionAtomic, item: &ItemsPath) -> io::Result<AtomicStep> {
    match action {
        ActionAtomic::Create => {
            check(item)?;
            prepare(item)
        }
        ActionAtomic::Modify => {
            check(item)?;
            prepare(item)
        }
        ActionAtomic::Rollback => Ok(AtomicStep::Waiting),
        ActionAtomic::Finish => Err(io::ErrorKind::InvalidInput.into()),
    }
}

/// A backup or a temp file already there is not this batch's: left by an
/// earlier one, or put there by hand. Starting over a backup could put stale
/// content back in place of the user's file, and writing over a temp file
/// would remove a file the batch did not create.
fn check(item: &ItemsPath) -> io::Result<()> {
    for leftover in [&item.backup_path, &item.temp_path] {
        if leftover.try_exists()? {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{} is in the way", leftover.display()),
            ));
        }
    }
    Ok(())
}
