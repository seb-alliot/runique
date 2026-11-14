use crate::atomic::{
    disk,
    step::{done, placed, prepared, replaced, waiting},
    struct_atomic::{ActionAtomic, AtomicStep, ItemsPath, PathManager},
};
use std::{io, path::Path};

/// Waiting → Prepared → Placed or Replaced → Done.
const PASSES: usize = 3;

/// Sends the file to the code of its current step. A failure names the file.
fn dispatch(action: &ActionAtomic, item: &ItemsPath) -> io::Result<AtomicStep> {
    let outcome = match item.step {
        AtomicStep::Waiting => waiting::run(action, item),
        AtomicStep::Prepared => prepared::run(action, item),
        AtomicStep::Replaced => replaced::run(action, item),
        AtomicStep::Placed => placed::run(action, item),
        AtomicStep::Done => done::run(action, item),
    };
    outcome.map_err(|error| named(&item.path, error))
}

/// The same error, same kind, with the path it happened on in front.
fn named(path: &Path, error: io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("{}: {error}", path.display()))
}

impl PathManager {
    /// Creates the missing parent directory of every file of the batch.
    fn create_dirs(&mut self) -> io::Result<()> {
        for item in self.items.values() {
            if let Some(parent) = item.path.parent() {
                disk::create_dirs(parent, &mut self.created_dirs)
                    .map_err(|error| named(parent, error))?;
            }
        }
        Ok(())
    }

    /// Moves every file one step at a time until the whole batch is Done.
    fn advance(&mut self, action: ActionAtomic) -> io::Result<()> {
        self.create_dirs()?;
        for _ in 0..PASSES {
            self.apply(&action)?;
        }
        if self.all_done() {
            Ok(())
        } else {
            Err(io::ErrorKind::InvalidInput.into())
        }
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
        for item in self.items.values_mut().rev() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomic::{struct_atomic::SourceItems, tests::TempDir};
    use std::fs;

    /// The file `name` of `dir`, declared at `step` whatever the disk holds.
    fn item(dir: &TempDir, name: &str, step: AtomicStep) -> ItemsPath {
        ItemsPath {
            path: dir.join(name),
            temp_path: dir.join(&format!("{name}.runique-tmp")),
            backup_path: dir.join(&format!("{name}.runique-bak")),
            private_file: false,
            source: SourceItems::Bytes(b"new".to_vec()),
            step,
        }
    }

    fn manager(items: Vec<ItemsPath>) -> PathManager {
        PathManager {
            items: items
                .into_iter()
                .map(|item| (item.path.clone(), item))
                .collect(),
            created_dirs: Vec::new(),
        }
    }

    // A directory where a placed file is expected: removing it as a file fails,
    // which is how a rollback is made to fail here.
    #[test]
    fn a_rollback_goes_on_past_a_failure_and_a_second_pass_finishes_it() {
        let dir = TempDir::new();
        fs::write(dir.join("a.txt"), "new").unwrap();
        fs::create_dir(dir.join("stuck")).unwrap();
        fs::write(dir.join("c.txt"), "new").unwrap();
        let mut manager = manager(vec![
            item(&dir, "a.txt", AtomicStep::Placed),
            item(&dir, "stuck", AtomicStep::Placed),
            item(&dir, "c.txt", AtomicStep::Placed),
        ]);

        assert!(manager.rollback().is_err());
        assert_eq!(dir.entries(), ["stuck"]);

        fs::remove_dir(dir.join("stuck")).unwrap();
        manager.rollback().unwrap();

        assert!(dir.entries().is_empty());
        assert!(
            manager
                .items
                .values()
                .all(|item| matches!(item.step, AtomicStep::Waiting))
        );
    }

    // The batch fails on the leftover backup, then its rollback fails twice on
    // `stuck`: the caller gets the rollback's error, not the one that started it.
    #[test]
    fn a_rollback_failing_twice_is_what_run_reports() {
        let dir = TempDir::new();
        fs::create_dir(dir.join("stuck")).unwrap();
        fs::write(dir.join("b.txt.runique-bak"), "stale").unwrap();
        let manager = manager(vec![
            item(&dir, "stuck", AtomicStep::Placed),
            item(&dir, "b.txt", AtomicStep::Waiting),
        ]);

        let error = manager.run(ActionAtomic::Create).unwrap_err();

        assert_ne!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(dir.entries(), ["b.txt.runique-bak", "stuck"]);
    }

    #[test]
    fn a_rollback_that_succeeds_leaves_the_first_error_to_run() {
        let dir = TempDir::new();
        fs::write(dir.join("a.txt"), "new").unwrap();
        fs::write(dir.join("b.txt.runique-bak"), "stale").unwrap();
        let manager = manager(vec![
            item(&dir, "a.txt", AtomicStep::Placed),
            item(&dir, "b.txt", AtomicStep::Waiting),
        ]);

        let error = manager.run(ActionAtomic::Create).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(dir.entries(), ["b.txt.runique-bak"]);
    }
}
