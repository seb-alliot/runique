use crate::atomic::{
    action::rollback,
    struct_atomic::{ActionAtomic, AtomicStep, ItemsPath},
};
use std::io;

pub(in crate::atomic) fn run(action: &ActionAtomic, item: &ItemsPath) -> io::Result<AtomicStep> {
    match action {
        ActionAtomic::Create => Err(io::ErrorKind::InvalidInput.into()),
        ActionAtomic::Modify => Ok(AtomicStep::Done),
        ActionAtomic::Rollback => rollback::restore(item),
        ActionAtomic::Finish => Err(io::ErrorKind::InvalidInput.into()),
    }
}
