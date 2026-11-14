use crate::atomic::{
    disk,
    step::{done, placed, prepared, replaced, waiting},
    struct_atomic::{ActionAtomic, AtomicStep, ItemsPath, PathManager},
};
use std::io;

/// Send the file to the code of its current step.
fn dispatch(action: &ActionAtomic, item: &ItemsPath) -> io::Result<AtomicStep> {
    match item.step {
        AtomicStep::Waiting => waiting::run(action, item),
        AtomicStep::Prepared => prepared::run(action, item),
        AtomicStep::Replaced => replaced::run(action, item),
        AtomicStep::Placed => placed::run(action, item),
        AtomicStep::Done => done::run(action, item),
    }
}

impl PathManager {
    /// Creates the missing parent directory of every file of the batch.
    fn create_dirs(&mut self) -> io::Result<()> {
        for item in self.items.values() {
            if let Some(parent) = item.path.parent() {
                disk::create_dirs(parent, &mut self.created_dirs)?;
            }
        }
        Ok(())
    }

    /// Moves every file one step at a time until the whole batch is Done.
    fn advance(&mut self, action: ActionAtomic) -> io::Result<()> {
        self.create_dirs()?;
        while !self.all_done() {
            self.apply(&action)?;
        }
        Ok(())
    }

    fn all_done(&self) -> bool {
        self.items
            .values()
            .all(|item| matches!(item.step, AtomicStep::Done))
    }

    fn apply(&mut self, action: &ActionAtomic) -> io::Result<()> {
        for item in self.items.values_mut() {
            item.step = dispatch(action, item)?;
        }
        Ok(())
    }

    /// Run the batch: any error undoes everything, success ends with the cleanup.
    pub(super) fn run(mut self, action: ActionAtomic) -> io::Result<()> {
        if let Err(error) = self.advance(action) {
            self.rollback().or_else(|_| self.rollback())?;
            return Err(error);
        }
        self.apply(&ActionAtomic::Finish)
    }

    /// On erollback pass over every file. A failure does not stop the others.
    fn rollback(&mut self) -> io::Result<()> {
        let mut outcome = Ok(());
        for item in self.items.values_mut() {
            match dispatch(&ActionAtomic::Rollback, item) {
                Ok(step) => item.step = step,
                Err(error) => outcome = Err(error),
            }
        }
        for dir in self.created_dirs.iter().rev() {
            if let Err(error) = disk::remove_dir(dir) {
                outcome = Err(error);
            }
        }
        outcome
    }
}
