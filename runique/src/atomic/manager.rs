use crate::atomic::{
    disk,
    journal::{self, Journal},
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
    /// Creates the missing parent directory of every file of the batch, and
    /// the journal's folder first.
    fn create_dirs(&mut self) -> io::Result<()> {
        if let Some(dir) = &self.journal {
            disk::create_dirs(dir, &mut self.created_dirs).map_err(|error| named(dir, error))?;
        }
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
        self.record()?;
        for _ in 0..PASSES {
            self.apply(&action)?;
        }
        if !self.all_done() {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        self.sync_dirs()?;
        self.mark_placed()
    }

    /// Writes the journal before any file is touched. An empty batch has
    /// nothing a cut could interrupt.
    fn record(&self) -> io::Result<()> {
        match &self.journal {
            Some(dir) if !self.items.is_empty() => journal::write(dir, &Journal::of(dir, self)?)
                .map_err(|error| named(dir, error)),
            Some(_) | None => Ok(()),
        }
    }

    /// The point of no return: from here a cut finishes the batch.
    fn mark_placed(&self) -> io::Result<()> {
        match &self.journal {
            Some(dir) if !self.items.is_empty() => {
                journal::mark_placed(dir).map_err(|error| named(dir, error))
            }
            Some(_) | None => Ok(()),
        }
    }

    /// Makes the files just placed survive a power loss: one sync for each
    /// directory the batch wrote into.
    fn sync_dirs(&self) -> io::Result<()> {
        let mut synced: Vec<&Path> = Vec::new();
        for item in self.items.values() {
            let dir = match item.path.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => parent,
                Some(_) | None => Path::new("."),
            };
            if !synced.contains(&dir) {
                disk::sync_dir(dir).map_err(|error| named(dir, error))?;
                synced.push(dir);
            }
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
            self.undo()?;
            return Err(error);
        }
        self.finish()
    }

    /// The final cleanup, then the journal: removed last, so that a cut in the
    /// middle finds it and finishes.
    pub(super) fn finish(&mut self) -> io::Result<()> {
        self.apply(&ActionAtomic::Finish)?;
        match &self.journal {
            Some(dir) => journal::remove(dir).map_err(|error| named(dir, error)),
            None => Ok(()),
        }
    }

    /// Undoes the whole batch, with a second pass when the first one fails.
    pub(super) fn undo(&mut self) -> io::Result<()> {
        self.rollback().or_else(|_| self.rollback())
    }

    /// One rollback pass over every file. A failure does not stop the others.
    /// The journal goes once every file is back, and stays otherwise: the next
    /// start then tries again.
    fn rollback(&mut self) -> io::Result<()> {
        let mut outcome = Ok(());
        for item in self.items.values_mut().rev() {
            match dispatch(&ActionAtomic::Rollback, item) {
                Ok(step) => item.step = step,
                Err(error) => outcome = Err(error),
            }
        }
        // The folders first, the journal after them: a cut in between finds
        // the journal and takes away what is left. Only the folders the
        // journal itself sits in have to wait for it to be gone.
        let holds_journal = |dir: &Path| {
            self.journal
                .as_deref()
                .is_some_and(|journal| journal.starts_with(dir))
        };
        for dir in self.created_dirs.iter().rev() {
            if !holds_journal(dir)
                && let Err(error) = disk::remove_dir(dir)
            {
                outcome = Err(error);
            }
        }
        if let Some(dir) = &self.journal
            && outcome.is_ok()
        {
            outcome = journal::remove(dir).map_err(|error| named(dir, error));
        }
        for dir in self.created_dirs.iter().rev() {
            if holds_journal(dir)
                && let Err(error) = disk::remove_dir(dir)
            {
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
            journal: None,
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

    // A directory where the backup is expected: the cleanup can't remove it.
    // Every file is in place by then, so nothing is undone.
    #[test]
    fn a_cleanup_that_fails_is_reported_and_undoes_nothing() {
        let dir = TempDir::new();
        fs::write(dir.join("a.txt"), "new").unwrap();
        fs::create_dir(dir.join("a.txt.runique-bak")).unwrap();
        let manager = manager(vec![item(&dir, "a.txt", AtomicStep::Done)]);

        let error = manager.run(ActionAtomic::Modify).unwrap_err();

        assert!(error.to_string().contains("a.txt"), "{error}");
        assert_eq!(fs::read_to_string(dir.join("a.txt")).unwrap(), "new");
        assert_eq!(dir.entries(), ["a.txt", "a.txt.runique-bak"]);
    }

    // A folder that can't be opened can't be synced: the batch is undone as for
    // any other failure. (Its own file can't be reached either, so the rollback
    // fails there too and that is the error reported.)
    #[cfg(unix)]
    #[test]
    fn a_folder_that_cannot_be_synced_undoes_the_batch() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new();
        fs::write(dir.join("a.txt"), "new").unwrap();
        fs::create_dir(dir.join("locked")).unwrap();
        fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o000)).unwrap();
        let manager = manager(vec![
            item(&dir, "a.txt", AtomicStep::Done),
            item(&dir, "locked/b.txt", AtomicStep::Done),
        ]);

        let outcome = manager.run(ActionAtomic::Create);
        fs::set_permissions(dir.join("locked"), fs::Permissions::from_mode(0o755)).unwrap();

        let error = outcome.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert!(error.to_string().contains("locked"), "{error}");
        assert_eq!(dir.entries(), ["locked"]);
    }

    // ── A cut at every stage, repaired from the journal the batch wrote ──────

    /// A batch with a journal on `a.txt` (there before, "original") and
    /// `sub/b.txt` (new, in a folder to create), run up to `passes` passes.
    fn cut_after(dir: &TempDir, passes: usize) -> PathManager {
        fs::write(dir.join("a.txt"), "original").unwrap();
        let mut manager = manager(vec![
            item(dir, "a.txt", AtomicStep::Waiting),
            item(dir, "sub/b.txt", AtomicStep::Waiting),
        ]);
        manager.journal = Some(dir.path().to_path_buf());
        manager.create_dirs().unwrap();
        manager.record().unwrap();
        for _ in 0..passes {
            manager.apply(&ActionAtomic::Modify).unwrap();
        }
        manager
    }

    // The manager is dropped where the power would go: nothing of it is left
    // but the disk. Whatever the pass, the repair gives the project back.
    #[test]
    fn a_cut_before_every_file_is_in_place_is_undone_from_the_journal() {
        for passes in 0..=PASSES {
            let dir = TempDir::new();
            drop(cut_after(&dir, passes));
            assert!(dir.join(".runique-journal").exists(), "pass {passes}");

            assert!(journal::recover(dir.path()).unwrap(), "pass {passes}");

            assert_eq!(
                fs::read_to_string(dir.join("a.txt")).unwrap(),
                "original",
                "pass {passes}"
            );
            assert_eq!(dir.entries(), ["a.txt"], "pass {passes}");
            assert!(!journal::recover(dir.path()).unwrap(), "nothing left to repair");
        }
    }

    // Past the point of no return the batch is finished instead: the cut may
    // have come before the cleanup, or after it removed one backup.
    #[test]
    fn a_cut_once_every_file_is_in_place_is_finished_from_the_journal() {
        for cleaned_one in [false, true] {
            let dir = TempDir::new();
            let mut manager = cut_after(&dir, PASSES);
            manager.mark_placed().unwrap();
            if cleaned_one {
                let first = manager.items.values_mut().next().unwrap();
                first.step = dispatch(&ActionAtomic::Finish, first).unwrap();
            }
            drop(manager);

            assert!(journal::recover(dir.path()).unwrap());

            assert_eq!(fs::read_to_string(dir.join("a.txt")).unwrap(), "new");
            assert_eq!(fs::read_to_string(dir.join("sub/b.txt")).unwrap(), "new");
            assert_eq!(dir.entries(), ["a.txt", "sub", "sub/b.txt"]);
        }
    }

    // The cut comes during the repair itself: started again, it ends the same.
    #[test]
    fn a_repair_cut_short_is_started_again() {
        let dir = TempDir::new();
        let mut manager = cut_after(&dir, 2);
        let last = manager.items.values_mut().next_back().unwrap();
        last.step = dispatch(&ActionAtomic::Rollback, last).unwrap();
        drop(manager);

        assert!(journal::recover(dir.path()).unwrap());

        assert_eq!(fs::read_to_string(dir.join("a.txt")).unwrap(), "original");
        assert_eq!(dir.entries(), ["a.txt"]);
    }

    // ── A cut after every single step of a batch, then the command again ─────

    use crate::atomic::struct_atomic::AtomicBuilder;

    /// Everything under `dir`: each folder, and each file with what it holds.
    fn disk(dir: &TempDir) -> Vec<(String, Option<String>)> {
        dir.entries()
            .into_iter()
            .map(|entry| {
                let content = fs::read_to_string(dir.join(&entry)).ok();
                (entry, content)
            })
            .collect()
    }

    /// One thing the batch does: a file moved one step on, the point of no
    /// return, a file cleaned up, the journal removed.
    enum Event {
        Step(usize),
        Mark,
        Clean(usize),
        Forget,
    }

    /// Runs the batch of `files` (name, new content) the way `run` does, and
    /// stops after `cut_at` events, handing back the batch as it then stands.
    fn stopped_at(
        dir: &TempDir,
        action: &ActionAtomic,
        files: &[(&str, &str)],
        cut_at: usize,
    ) -> PathManager {
        let items = files
            .iter()
            .map(|(name, content)| {
                let mut file = item(dir, name, AtomicStep::Waiting);
                file.source = SourceItems::Bytes(content.as_bytes().to_vec());
                file
            })
            .collect();
        let mut manager = manager(items);
        manager.journal = Some(dir.path().to_path_buf());
        manager.create_dirs().unwrap();
        manager.record().unwrap();

        let count = files.len();
        let events = (0..PASSES)
            .flat_map(|_| (0..count).map(Event::Step))
            .chain([Event::Mark])
            .chain((0..count).map(Event::Clean))
            .chain([Event::Forget]);
        for event in events.take(cut_at) {
            match event {
                Event::Step(index) => {
                    let (_, file) = manager.items.get_index_mut(index).unwrap();
                    file.step = dispatch(action, file).unwrap();
                }
                Event::Mark => {
                    manager.sync_dirs().unwrap();
                    manager.mark_placed().unwrap();
                }
                Event::Clean(index) => {
                    let (_, file) = manager.items.get_index_mut(index).unwrap();
                    file.step = dispatch(&ActionAtomic::Finish, file).unwrap();
                }
                Event::Forget => journal::remove(dir.path()).unwrap(),
            }
        }
        manager
    }

    /// The same, the manager dropped: nothing is left of the batch but the disk.
    fn cut_at(dir: &TempDir, action: &ActionAtomic, files: &[(&str, &str)], cut_at: usize) {
        drop(stopped_at(dir, action, files, cut_at));
    }

    /// The whole batch through the builder, as a command would run it.
    fn whole_batch(dir: &TempDir, modify: bool, files: &[(&str, &str)]) -> io::Result<()> {
        let mut batch = if modify {
            AtomicBuilder::modify()
        } else {
            AtomicBuilder::create()
        }
        .journal(dir.path());
        for (name, content) in files {
            batch = batch.add(
                dir.join(name),
                SourceItems::Bytes(content.as_bytes().to_vec()),
                false,
            )?;
        }
        batch.start()
    }

    // Files that read alike under different names, on purpose: the repair tells
    // its own files by their content. Two originals hold the same text, one is
    // replaced by that very text, and three new files share another.
    const REPLACED: [(&str, &str); 5] = [
        ("same_1.txt", "identical"),
        ("same_2.txt", "replaced"),
        ("copy_a.txt", "replaced"),
        ("sub/copy_b.txt", "replaced"),
        ("sub/deep/c.txt", "other"),
    ];

    fn originals(dir: &TempDir) {
        fs::write(dir.join("same_1.txt"), "identical").unwrap();
        fs::write(dir.join("same_2.txt"), "identical").unwrap();
    }

    // The power goes after each of the 22 things the batch does, one at a time.
    // Before the point of no return the repair gives back the project exactly
    // as it was; after it, the batch is finished. Either way nothing of the
    // batch's own is left, and the command run again ends on the same disk.
    #[test]
    fn a_replacement_cut_after_any_step_is_repaired_then_runs_again() {
        let before = {
            let dir = TempDir::new();
            originals(&dir);
            disk(&dir)
        };
        let after = {
            let dir = TempDir::new();
            originals(&dir);
            whole_batch(&dir, true, &REPLACED).unwrap();
            disk(&dir)
        };
        let point_of_no_return = PASSES * REPLACED.len();
        let last = point_of_no_return + 1 + REPLACED.len() + 1;

        for cut in 0..=last {
            let dir = TempDir::new();
            originals(&dir);
            cut_at(&dir, &ActionAtomic::Modify, &REPLACED, cut);

            let repaired = journal::recover(dir.path()).unwrap();

            assert_eq!(repaired, cut < last, "cut {cut}");
            let expected = if cut <= point_of_no_return {
                &before
            } else {
                &after
            };
            assert_eq!(&disk(&dir), expected, "cut {cut}");

            whole_batch(&dir, true, &REPLACED).unwrap();
            assert_eq!(disk(&dir), after, "cut {cut}, run again");
        }
    }

    // The same for a creation, every file new and all three alike. Undone, the
    // disk is empty and the command creates the files; finished, they are
    // there and the command, run again, refuses to touch them.
    #[test]
    fn a_creation_cut_after_any_step_is_repaired_then_runs_again() {
        let files = [("a.txt", "same"), ("sub/b.txt", "same"), ("sub/c.txt", "same")];
        let after = {
            let dir = TempDir::new();
            whole_batch(&dir, false, &files).unwrap();
            disk(&dir)
        };
        let point_of_no_return = PASSES * files.len();
        let last = point_of_no_return + 1 + files.len() + 1;

        for cut in 0..=last {
            let dir = TempDir::new();
            cut_at(&dir, &ActionAtomic::Create, &files, cut);

            let repaired = journal::recover(dir.path()).unwrap();

            assert_eq!(repaired, cut < last, "cut {cut}");
            if cut <= point_of_no_return {
                assert!(disk(&dir).is_empty(), "cut {cut}");
                whole_batch(&dir, false, &files).unwrap();
            } else {
                assert_eq!(disk(&dir), after, "cut {cut}");
                let again = whole_batch(&dir, false, &files).unwrap_err();
                assert_eq!(again.kind(), io::ErrorKind::AlreadyExists, "cut {cut}");
            }
            assert_eq!(disk(&dir), after, "cut {cut}, run again");
        }
    }

    // ── Two incidents in a row ───────────────────────────────────────────────

    const POINT_OF_NO_RETURN: usize = PASSES * REPLACED.len();
    /// Past the last thing a batch of `REPLACED` does: the journal is gone.
    const FINISHED: usize = POINT_OF_NO_RETURN + 1 + REPLACED.len() + 1;

    fn before_and_after() -> (Vec<(String, Option<String>)>, Vec<(String, Option<String>)>) {
        let dir = TempDir::new();
        originals(&dir);
        let before = disk(&dir);
        whole_batch(&dir, true, &REPLACED).unwrap();
        (before, disk(&dir))
    }

    /// Starts the repair of `dir` and stops it dead after `steps` of the things
    /// it does. Returns whether it was indeed cut before its end.
    fn repair_cut_at(dir: &TempDir, steps: usize) -> bool {
        let Some(mut repair) = journal::examine(dir.path()).unwrap() else {
            return false;
        };
        let mut left = steps;
        let mut goes_on = || {
            let more = left > 0;
            left = left.saturating_sub(1);
            more
        };
        for backup in &repair.spare_backups {
            if !goes_on() {
                return true;
            }
            disk::remove(backup).unwrap();
        }
        if repair.placed {
            for file in repair.manager.items.values_mut() {
                if !goes_on() {
                    return true;
                }
                file.step = dispatch(&ActionAtomic::Finish, file).unwrap();
            }
        } else {
            for file in repair.manager.items.values_mut().rev() {
                if !goes_on() {
                    return true;
                }
                file.step = dispatch(&ActionAtomic::Rollback, file).unwrap();
            }
            for created in repair.manager.created_dirs.iter().rev() {
                if !goes_on() {
                    return true;
                }
                disk::remove_dir(created).unwrap();
            }
        }
        if !goes_on() {
            return true;
        }
        journal::remove(dir.path()).unwrap();
        false
    }

    // The batch is cut after any of its steps, then the repair itself is cut
    // after any of its own. Started again, the repair ends on the very disk a
    // repair left alone would have given.
    #[test]
    fn a_repair_cut_after_any_of_its_steps_is_started_again() {
        let (before, after) = before_and_after();

        for cut in 0..FINISHED {
            for steps in 0.. {
                let dir = TempDir::new();
                originals(&dir);
                cut_at(&dir, &ActionAtomic::Modify, &REPLACED, cut);

                let interrupted = repair_cut_at(&dir, steps);
                journal::recover(dir.path()).unwrap();

                let expected = if cut <= POINT_OF_NO_RETURN {
                    &before
                } else {
                    &after
                };
                assert_eq!(&disk(&dir), expected, "cut {cut}, repair cut after {steps}");
                if !interrupted {
                    break;
                }
            }
        }
    }

    // A cut, a repair, the command again, and the power goes a second time.
    #[test]
    fn a_second_cut_after_a_repair_is_repaired_too() {
        let (_, after) = before_and_after();

        for first in [0, 3, 8, 12, 15, 16, 19, 21] {
            for second in [0, 4, 9, 15, 16, 20] {
                let dir = TempDir::new();
                originals(&dir);
                cut_at(&dir, &ActionAtomic::Modify, &REPLACED, first);
                journal::recover(dir.path()).unwrap();
                let start = disk(&dir);

                cut_at(&dir, &ActionAtomic::Modify, &REPLACED, second);
                assert!(journal::recover(dir.path()).unwrap());

                let expected = if second <= POINT_OF_NO_RETURN {
                    &start
                } else {
                    &after
                };
                assert_eq!(&disk(&dir), expected, "cuts {first} then {second}");
                whole_batch(&dir, true, &REPLACED).unwrap();
                assert_eq!(disk(&dir), after, "cuts {first} then {second}, run again");
            }
        }
    }

    // No cut here: the batch fails, and its own undo fails too (a folder turned
    // read-only keeps one file from being removed). The journal stays, and the
    // next start finishes what the undo could not.
    #[cfg(unix)]
    #[test]
    fn an_undo_that_fails_keeps_the_journal_for_the_next_start() {
        use std::os::unix::fs::PermissionsExt;
        let (before, _) = before_and_after();
        let dir = TempDir::new();
        originals(&dir);
        let mut manager = stopped_at(&dir, &ActionAtomic::Modify, &REPLACED, 10);
        let locked = dir.join("sub/deep");
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).unwrap();

        let undone = manager.undo();
        fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
        drop(manager);

        assert!(undone.is_err());
        assert!(dir.join(".runique-journal").exists());
        assert!(journal::recover(dir.path()).unwrap());
        assert_eq!(disk(&dir), before);
    }

    // Every file is in place and the cleanup fails on one backup it can't
    // remove (a directory by that name). The journal stays, marked: the next
    // start finishes the cleanup once the way is clear, and undoes nothing.
    #[test]
    fn a_cleanup_that_fails_keeps_the_journal_for_the_next_start() {
        let (_, after) = before_and_after();
        let dir = TempDir::new();
        originals(&dir);
        let mut manager = stopped_at(
            &dir,
            &ActionAtomic::Modify,
            &REPLACED,
            POINT_OF_NO_RETURN + 1,
        );
        fs::create_dir(dir.join("copy_a.txt.runique-bak")).unwrap();

        assert!(manager.finish().is_err());
        drop(manager);
        assert!(dir.join(".runique-journal").exists());
        assert!(journal::recover(dir.path()).is_err(), "still in the way");
        assert!(dir.join(".runique-journal").exists());

        fs::remove_dir(dir.join("copy_a.txt.runique-bak")).unwrap();
        assert!(journal::recover(dir.path()).unwrap());
        assert_eq!(disk(&dir), after);
    }

    // The power went while the journal itself was being written. The first
    // time, only its temp file is there: no batch had started. At the mark, the
    // journal still says "not in place": the batch is undone.
    #[test]
    fn a_cut_while_the_journal_was_being_written_is_no_obstacle() {
        let (before, after) = before_and_after();

        let dir = TempDir::new();
        originals(&dir);
        fs::write(dir.join(".runique-journal.runique-tmp"), "{ \"placed\": fal").unwrap();
        assert!(!journal::recover(dir.path()).unwrap());
        whole_batch(&dir, true, &REPLACED).unwrap();
        assert_eq!(disk(&dir), after);

        let dir = TempDir::new();
        originals(&dir);
        cut_at(&dir, &ActionAtomic::Modify, &REPLACED, POINT_OF_NO_RETURN);
        fs::write(dir.join(".runique-journal.runique-tmp"), "{ \"placed\": tru").unwrap();
        assert!(journal::recover(dir.path()).unwrap());
        assert_eq!(disk(&dir), before);
    }

    // After the cut, the developer removed one of the batch's own files.
    #[test]
    fn a_temp_file_or_a_backup_removed_by_hand_after_the_cut() {
        // The temp file of a new file: nothing tells that file from a stranger.
        // The temp file of a replaced one: its content is no longer vouched for.
        for (removed, named) in [
            ("copy_a.txt.runique-tmp", "copy_a.txt"),
            ("same_2.txt.runique-tmp", "same_2.txt"),
        ] {
            let dir = TempDir::new();
            originals(&dir);
            cut_at(&dir, &ActionAtomic::Modify, &REPLACED, 10);
            fs::remove_file(dir.join(removed)).unwrap();
            let on_disk = disk(&dir);

            let error = journal::recover(dir.path()).unwrap_err();

            assert_eq!(error.kind(), io::ErrorKind::InvalidData, "{removed}");
            assert!(error.to_string().contains(named), "{error}");
            assert_eq!(disk(&dir), on_disk, "{removed}");
        }

        // The backup of a replaced file: the original is gone for good, by
        // their hand. The rest is undone; that file keeps what the batch placed.
        let dir = TempDir::new();
        originals(&dir);
        cut_at(&dir, &ActionAtomic::Modify, &REPLACED, 10);
        fs::remove_file(dir.join("same_2.txt.runique-bak")).unwrap();

        assert!(journal::recover(dir.path()).unwrap());

        assert_eq!(
            disk(&dir),
            [
                ("same_1.txt".to_string(), Some("identical".to_string())),
                ("same_2.txt".to_string(), Some("replaced".to_string())),
            ]
        );
    }

    // The repair went through, and the command run after it fails for a reason
    // of its own: it leaves the project as the repair gave it back.
    #[cfg(unix)]
    #[test]
    fn a_batch_that_fails_right_after_a_repair_leaves_the_project_as_it_was() {
        let (before, _) = before_and_after();
        let dir = TempDir::new();
        originals(&dir);
        cut_at(&dir, &ActionAtomic::Modify, &REPLACED, 8);
        assert!(journal::recover(dir.path()).unwrap());
        fs::create_dir(dir.join("blocked.txt")).unwrap();
        let mut files = REPLACED.to_vec();
        files.push(("blocked.txt", "never written"));

        assert!(whole_batch(&dir, true, &files).is_err());

        fs::remove_dir(dir.join("blocked.txt")).unwrap();
        assert_eq!(disk(&dir), before);
    }

    // The power went before the journal existed, while the folders were being
    // made: they are left, empty, and no journal says so. They don't get in
    // the way of the batch run again.
    #[test]
    fn folders_left_empty_by_a_cut_before_the_journal_do_not_stop_the_batch() {
        let (_, after) = before_and_after();
        let dir = TempDir::new();
        originals(&dir);
        fs::create_dir_all(dir.join("sub/deep")).unwrap();

        assert!(!journal::recover(dir.path()).unwrap());
        whole_batch(&dir, true, &REPLACED).unwrap();

        assert_eq!(disk(&dir), after);
    }

    // ── After the cut, a file removed by hand before the command runs again ──

    // Every file is placed or replaced when the power goes (10 steps in).
    // Removing one the batch created, one it replaced, or a whole folder it
    // made takes nothing the repair needs: the project comes back as it was.
    #[test]
    fn a_file_of_the_batch_removed_by_hand_does_not_stop_the_repair() {
        let (before, _) = before_and_after();
        let removals: [&dyn Fn(&TempDir); 4] = [
            &|dir| fs::remove_file(dir.join("copy_a.txt")).unwrap(),
            &|dir| fs::remove_file(dir.join("same_2.txt")).unwrap(),
            &|dir| fs::remove_dir_all(dir.join("sub/deep")).unwrap(),
            &|dir| fs::remove_dir_all(dir.join("sub")).unwrap(),
        ];
        for (index, remove) in removals.iter().enumerate() {
            let dir = TempDir::new();
            originals(&dir);
            cut_at(&dir, &ActionAtomic::Modify, &REPLACED, 10);
            remove(&dir);

            assert!(journal::recover(dir.path()).unwrap(), "removal {index}");

            assert_eq!(disk(&dir), before, "removal {index}");
        }
    }

    // The cut came while the temp files were being written: `same_2.txt` was
    // not reached, so no backup holds it. Removed by hand, it is gone for good:
    // the repair says so and touches nothing.
    #[test]
    fn an_original_removed_by_hand_before_it_was_backed_up_stops_the_repair() {
        let dir = TempDir::new();
        originals(&dir);
        cut_at(&dir, &ActionAtomic::Modify, &REPLACED, 3);
        fs::remove_file(dir.join("same_2.txt")).unwrap();
        let on_disk = disk(&dir);

        let error = journal::recover(dir.path()).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error.to_string().contains("same_2.txt"), "{error}");
        assert_eq!(disk(&dir), on_disk);
    }

    // The journal itself was removed: nothing says anymore whose the leftover
    // files are. The command is refused on the first one and changes nothing,
    // whether it replaces or creates.
    #[test]
    fn a_journal_removed_by_hand_leaves_the_batch_refused() {
        for cut in [3, 10, 18] {
            let dir = TempDir::new();
            originals(&dir);
            cut_at(&dir, &ActionAtomic::Modify, &REPLACED, cut);
            fs::remove_file(dir.join(".runique-journal")).unwrap();
            let on_disk = disk(&dir);

            assert!(!journal::recover(dir.path()).unwrap(), "cut {cut}");
            let error = whole_batch(&dir, true, &REPLACED).unwrap_err();

            assert_eq!(error.kind(), io::ErrorKind::AlreadyExists, "cut {cut}");
            assert!(error.to_string().contains(".runique-"), "{error}");
            assert_eq!(disk(&dir), on_disk, "cut {cut}");
        }
    }

    // The journal is removed, then the leftovers too: with nothing of the
    // batch's own left, the command runs as on any project.
    #[test]
    fn a_project_cleared_by_hand_after_a_cut_runs_again() {
        let (_, after) = before_and_after();
        let dir = TempDir::new();
        originals(&dir);
        cut_at(&dir, &ActionAtomic::Modify, &REPLACED, 10);
        for entry in dir.entries() {
            if entry.contains(".runique-") {
                fs::remove_file(dir.join(&entry)).unwrap();
            }
        }

        whole_batch(&dir, true, &REPLACED).unwrap();

        assert_eq!(disk(&dir), after);
    }
}
