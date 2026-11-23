use crate::{
    action::rollback,
    struct_atomic::{ActionAtomic, AtomicStep, ItemsPath},
};
use std::io;

pub(crate) fn run(action: &ActionAtomic, item: &ItemsPath) -> io::Result<AtomicStep> {
    match action {
        ActionAtomic::Create => super::wrong_step(),
        ActionAtomic::Modify => Ok(AtomicStep::Done),
        ActionAtomic::Rollback => rollback::restore(item),
        ActionAtomic::Finish => super::wrong_step(),
    }
}
