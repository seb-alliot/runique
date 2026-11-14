use crate::atomic::{
    action::{create, modify},
    struct_atomic::{ActionAtomic, AtomicStep, ItemsPath},
};
use std::io;

/// Nothing has been done for this file yet.
pub(in crate::atomic) fn run(action: &ActionAtomic, item: &ItemsPath) -> io::Result<AtomicStep> {
    match action {
        ActionAtomic::Create => create::prepare(item),
        ActionAtomic::Modify => modify::prepare(item),
        ActionAtomic::Rollback => Ok(AtomicStep::Waiting),
        ActionAtomic::Finish => Err(io::ErrorKind::InvalidInput.into()),
    }
}
