use crate::{
    action::rollback,
    struct_atomic::{ActionAtomic, AtomicStep, ItemsPath},
};
use std::io;

/// The file is at `path`, and nothing was there before.
pub(crate) fn run(action: &ActionAtomic, item: &ItemsPath) -> io::Result<AtomicStep> {
    match action {
        ActionAtomic::Create => Ok(AtomicStep::Done),
        ActionAtomic::Modify => Ok(AtomicStep::Done),
        ActionAtomic::Rollback => rollback::remove(item),
        ActionAtomic::Finish => super::wrong_step(),
    }
}
