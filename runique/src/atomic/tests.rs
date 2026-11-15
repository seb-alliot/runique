//! Whole batches run through the builder's own entry points, then each cell of
//! the step × action table on a disk laid out by hand.
use super::{
    action::rollback,
    disk,
    journal::{self, Journal, JournalFile, recover},
    step::{done, placed, prepared, replaced, waiting},
    struct_atomic::{ActionAtomic, AtomicBuilder, AtomicStep, ItemsPath, SourceItems},
};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// A directory under the system temp dir, removed on drop, even when the test fails.
pub(crate) struct TempDir(PathBuf);

impl TempDir {
    pub(crate) fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("rq_atomic_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    pub(crate) fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }

    /// Every entry left under the directory, relative and sorted.
    pub(crate) fn entries(&self) -> Vec<String> {
        fn walk(root: &Path, dir: &Path, found: &mut Vec<String>) {
            for entry in fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                let relative = path.strip_prefix(root).unwrap().to_string_lossy();
                found.push(relative.replace('\\', "/"));
                if path.is_dir() {
                    walk(root, &path, found);
                }
            }
        }
        let mut found = Vec::new();
        walk(&self.0, &self.0, &mut found);
        found.sort();
        found
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn bytes(text: &str) -> SourceItems {
    SourceItems::Bytes(text.as_bytes().to_vec())
}

fn write(path: &Path, text: &str) {
    fs::write(path, text).unwrap();
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[cfg(unix)]
fn mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

/// The file `file.txt` of `dir`, with nothing on disk yet.
fn item(dir: &TempDir, source: SourceItems) -> ItemsPath {
    ItemsPath {
        path: dir.join("file.txt"),
        temp_path: dir.join("file.txt.runique-tmp"),
        backup_path: dir.join("file.txt.runique-bak"),
        private_file: false,
        source,
        step: AtomicStep::Waiting,
    }
}

/// Puts the disk in a given state: each `Some` is written, each `None` stays absent.
fn lay_out(item: &ItemsPath, path: Option<&str>, temp: Option<&str>, backup: Option<&str>) {
    for (target, text) in [
        (&item.path, path),
        (&item.temp_path, temp),
        (&item.backup_path, backup),
    ] {
        if let Some(text) = text {
            write(target, text);
        }
    }
}

fn refused(outcome: io::Result<AtomicStep>) -> bool {
    matches!(outcome, Err(error) if error.kind() == io::ErrorKind::InvalidInput)
}

fn already_exists<T>(outcome: io::Result<T>) -> bool {
    matches!(outcome, Err(error) if error.kind() == io::ErrorKind::AlreadyExists)
}

const PATH: &str = "file.txt";
const TEMP: &str = "file.txt.runique-tmp";
const BACKUP: &str = "file.txt.runique-bak";

// ── Whole batches ────────────────────────────────────────────────────────────

#[test]
fn a_create_batch_writes_every_file_and_nothing_else() {
    let dir = TempDir::new();
    AtomicBuilder::create()
        .add(dir.join("a.txt"), bytes("a"), false)
        .unwrap()
        .add(dir.join("sub/deep/b.txt"), bytes("b"), false)
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(read(&dir.join("a.txt")), "a");
    assert_eq!(read(&dir.join("sub/deep/b.txt")), "b");
    assert_eq!(
        dir.entries(),
        ["a.txt", "sub", "sub/deep", "sub/deep/b.txt"]
    );
}

#[test]
fn a_create_batch_meeting_an_existing_file_leaves_the_disk_as_it_was() {
    let dir = TempDir::new();
    write(&dir.join("taken.txt"), "mine");

    let outcome = AtomicBuilder::create()
        .add(dir.join("sub/new.txt"), bytes("new"), false)
        .unwrap()
        .add(dir.join("taken.txt"), bytes("theirs"), false)
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join("taken.txt")), "mine");
    assert_eq!(dir.entries(), ["taken.txt"]);
}

#[test]
fn a_directory_at_the_path_counts_as_an_existing_file() {
    let dir = TempDir::new();
    fs::create_dir(dir.join("taken")).unwrap();

    let outcome = AtomicBuilder::create()
        .add(dir.join("taken"), bytes("new"), false)
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(dir.entries(), ["taken"]);
}

#[test]
fn a_directory_that_was_already_there_survives_the_rollback() {
    let dir = TempDir::new();
    fs::create_dir(dir.join("keep")).unwrap();
    write(&dir.join("keep/taken.txt"), "mine");

    let outcome = AtomicBuilder::create()
        .add(dir.join("keep/new.txt"), bytes("new"), false)
        .unwrap()
        .add(dir.join("keep/made/deep.txt"), bytes("deep"), false)
        .unwrap()
        .add(dir.join("keep/taken.txt"), bytes("theirs"), false)
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join("keep/taken.txt")), "mine");
    assert_eq!(dir.entries(), ["keep", "keep/taken.txt"]);
}

#[test]
fn a_modify_batch_replaces_a_file_and_cleans_up_after_itself() {
    let dir = TempDir::new();
    write(&dir.join("old.txt"), "before");

    AtomicBuilder::modify()
        .add(dir.join("old.txt"), bytes("after"), false)
        .unwrap()
        .add(dir.join("new.txt"), bytes("fresh"), false)
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(read(&dir.join("old.txt")), "after");
    assert_eq!(read(&dir.join("new.txt")), "fresh");
    assert_eq!(dir.entries(), ["new.txt", "old.txt"]);
}

// A directory can be neither hard-linked nor copied as a file: the backup of
// `b.txt` fails once `a.txt` has already been replaced.
#[cfg(unix)]
#[test]
fn a_modify_batch_failing_halfway_puts_every_original_back() {
    let dir = TempDir::new();
    write(&dir.join("a.txt"), "before");
    fs::create_dir(dir.join("b.txt")).unwrap();

    let outcome = AtomicBuilder::modify()
        .add(dir.join("a.txt"), bytes("after"), false)
        .unwrap()
        .add(dir.join("b.txt"), bytes("new"), false)
        .unwrap()
        .start();

    assert!(outcome.is_err());
    assert_eq!(read(&dir.join("a.txt")), "before");
    assert!(dir.join("b.txt").is_dir());
    assert_eq!(dir.entries(), ["a.txt", "b.txt"]);
}

#[test]
fn a_leftover_backup_stops_the_batch_before_any_file_is_touched() {
    let dir = TempDir::new();
    write(&dir.join("first.txt"), "one");
    write(&dir.join("second.txt"), "two");
    write(&dir.join("second.txt.runique-bak"), "stale");

    let outcome = AtomicBuilder::modify()
        .add(dir.join("first.txt"), bytes("new one"), false)
        .unwrap()
        .add(dir.join("second.txt"), bytes("new two"), false)
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join("first.txt")), "one");
    assert_eq!(read(&dir.join("second.txt")), "two");
    assert_eq!(read(&dir.join("second.txt.runique-bak")), "stale");
    assert_eq!(
        dir.entries(),
        ["first.txt", "second.txt", "second.txt.runique-bak"]
    );
}

// A temp file left by an interrupted batch may be a hard link to the file
// itself: writing into it would change the file before any backup exists.
#[test]
fn a_stale_temp_file_is_never_written_through() {
    let dir = TempDir::new();
    write(&dir.join(PATH), "current");
    fs::hard_link(dir.join(PATH), dir.join(TEMP)).unwrap();

    let outcome = AtomicBuilder::create()
        .add(dir.join(PATH), bytes("new"), false)
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join(PATH)), "current");
}

#[test]
fn the_same_path_twice_is_refused_when_added() {
    let dir = TempDir::new();
    let outcome = AtomicBuilder::create()
        .add(dir.join("a.txt"), bytes("a"), false)
        .unwrap()
        .add(dir.join("a.txt"), bytes("again"), false);

    assert!(already_exists(outcome));
}

// `sub/../a.txt` is `a.txt` on disk but another key in the batch: the second
// one meets the file the first just placed, and the whole batch is undone.
#[test]
fn two_spellings_of_one_path_undo_the_whole_batch() {
    let dir = TempDir::new();
    fs::create_dir(dir.join("sub")).unwrap();

    let outcome = AtomicBuilder::create()
        .add(dir.join("a.txt"), bytes("first"), false)
        .unwrap()
        .add(dir.join("sub/../a.txt"), bytes("second"), false)
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(dir.entries(), ["sub"]);
}

#[test]
fn an_empty_batch_does_nothing() {
    let dir = TempDir::new();
    AtomicBuilder::create().start().unwrap();
    AtomicBuilder::modify().start().unwrap();
    assert!(dir.entries().is_empty());
}

#[test]
fn a_file_source_ends_up_at_the_path_and_stays_where_it_was() {
    let dir = TempDir::new();
    write(&dir.join("staged"), "upload");

    AtomicBuilder::modify()
        .add(
            dir.join("final.bin"),
            SourceItems::File(dir.join("staged")),
            false,
        )
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(read(&dir.join("final.bin")), "upload");
    assert_eq!(read(&dir.join("staged")), "upload");
    assert_eq!(dir.entries(), ["final.bin", "staged"]);
}

#[test]
fn a_file_source_in_a_create_batch_never_overwrites() {
    let dir = TempDir::new();
    write(&dir.join("staged"), "upload");
    write(&dir.join("final.bin"), "mine");

    let outcome = AtomicBuilder::create()
        .add(
            dir.join("final.bin"),
            SourceItems::File(dir.join("staged")),
            false,
        )
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join("final.bin")), "mine");
    assert_eq!(read(&dir.join("staged")), "upload");
    assert_eq!(dir.entries(), ["final.bin", "staged"]);
}

#[cfg(unix)]
#[test]
fn a_private_file_is_readable_by_its_owner_only() {
    let dir = TempDir::new();
    AtomicBuilder::create()
        .add(dir.join(".env"), bytes("SECRET_KEY=x"), true)
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(mode(&dir.join(".env")), 0o600);
    assert_eq!(dir.entries(), [".env"]);
}

// ── Waiting ──────────────────────────────────────────────────────────────────

#[test]
fn waiting_writes_the_temp_file_for_create_and_for_modify() {
    for action in [ActionAtomic::Create, ActionAtomic::Modify] {
        let dir = TempDir::new();
        let item = item(&dir, bytes("new"));

        assert!(matches!(
            waiting::run(&action, &item),
            Ok(AtomicStep::Prepared)
        ));
        assert_eq!(read(&item.temp_path), "new");
        assert_eq!(dir.entries(), [TEMP]);
    }
}

#[test]
fn waiting_has_nothing_to_undo() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("mine"), None, None);

    assert!(matches!(
        waiting::run(&ActionAtomic::Rollback, &item),
        Ok(AtomicStep::Waiting)
    ));
    assert_eq!(read(&item.path), "mine");
    assert_eq!(dir.entries(), [PATH]);
}

#[test]
fn waiting_refuses_to_start_over_a_leftover_backup() {
    for action in [ActionAtomic::Create, ActionAtomic::Modify] {
        let dir = TempDir::new();
        let item = item(&dir, bytes("new"));
        lay_out(&item, None, None, Some("stale"));

        assert!(already_exists(waiting::run(&action, &item)));
        assert_eq!(read(&item.backup_path), "stale");
        assert_eq!(dir.entries(), [BACKUP]);
    }
}

#[test]
fn waiting_links_a_file_source_without_touching_it() {
    for action in [ActionAtomic::Create, ActionAtomic::Modify] {
        let dir = TempDir::new();
        write(&dir.join("staged"), "upload");
        let item = item(&dir, SourceItems::File(dir.join("staged")));

        assert!(matches!(
            waiting::run(&action, &item),
            Ok(AtomicStep::Prepared)
        ));
        assert_eq!(read(&item.temp_path), "upload");
        assert_eq!(read(&dir.join("staged")), "upload");
        assert_eq!(dir.entries(), [TEMP, "staged"]);
    }
}

// ── Prepared ─────────────────────────────────────────────────────────────────

#[test]
fn prepared_create_places_the_file_and_keeps_the_temp_file() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, None, Some("new"), None);

    assert!(matches!(
        prepared::run(&ActionAtomic::Create, &item),
        Ok(AtomicStep::Placed)
    ));
    assert_eq!(read(&item.path), "new");
    assert_eq!(dir.entries(), [PATH, TEMP]);
}

#[test]
fn prepared_create_never_overwrites() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("mine"), Some("new"), None);

    assert!(already_exists(prepared::run(&ActionAtomic::Create, &item)));
    assert_eq!(read(&item.path), "mine");
    assert_eq!(dir.entries(), [PATH, TEMP]);
}

#[test]
fn prepared_modify_keeps_the_original_as_a_backup() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("old"), Some("new"), None);

    assert!(matches!(
        prepared::run(&ActionAtomic::Modify, &item),
        Ok(AtomicStep::Replaced)
    ));
    assert_eq!(read(&item.path), "new");
    assert_eq!(read(&item.backup_path), "old");
    assert_eq!(dir.entries(), [PATH, BACKUP, TEMP]);
}

#[test]
fn prepared_modify_without_an_original_only_places() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, None, Some("new"), None);

    assert!(matches!(
        prepared::run(&ActionAtomic::Modify, &item),
        Ok(AtomicStep::Placed)
    ));
    assert_eq!(read(&item.path), "new");
    assert_eq!(dir.entries(), [PATH, TEMP]);
}

#[test]
fn prepared_rollback_only_drops_the_temp_file_when_nothing_moved() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("mine"), Some("new"), None);

    assert!(matches!(
        prepared::run(&ActionAtomic::Rollback, &item),
        Ok(AtomicStep::Waiting)
    ));
    assert_eq!(read(&item.path), "mine");
    assert_eq!(dir.entries(), [PATH]);
}

// `replace` stopped right after the backup: the original may still be at
// `path`, or already gone.
#[test]
fn prepared_rollback_puts_the_original_back_after_an_interrupted_replace() {
    for original_still_there in [true, false] {
        let dir = TempDir::new();
        let item = item(&dir, bytes("new"));
        lay_out(
            &item,
            original_still_there.then_some("old"),
            Some("new"),
            Some("old"),
        );

        assert!(matches!(
            prepared::run(&ActionAtomic::Rollback, &item),
            Ok(AtomicStep::Waiting)
        ));
        assert_eq!(read(&item.path), "old");
        assert_eq!(dir.entries(), [PATH]);
    }
}

// ── Placed ───────────────────────────────────────────────────────────────────

#[test]
fn placed_is_done_without_touching_the_disk() {
    for action in [ActionAtomic::Create, ActionAtomic::Modify] {
        let dir = TempDir::new();
        let item = item(&dir, bytes("new"));
        lay_out(&item, Some("new"), Some("new"), None);

        assert!(matches!(placed::run(&action, &item), Ok(AtomicStep::Done)));
        assert_eq!(dir.entries(), [PATH, TEMP]);
    }
}

#[test]
fn placed_rollback_removes_the_file_it_placed() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("new"), Some("new"), None);

    assert!(matches!(
        placed::run(&ActionAtomic::Rollback, &item),
        Ok(AtomicStep::Waiting)
    ));
    assert!(dir.entries().is_empty());
}

// ── Replaced ─────────────────────────────────────────────────────────────────

#[test]
fn replaced_modify_is_done_without_touching_the_disk() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("new"), Some("new"), Some("old"));

    assert!(matches!(
        replaced::run(&ActionAtomic::Modify, &item),
        Ok(AtomicStep::Done)
    ));
    assert_eq!(dir.entries(), [PATH, BACKUP, TEMP]);
}

#[test]
fn replaced_rollback_puts_the_original_back() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("new"), Some("new"), Some("old"));

    assert!(matches!(
        replaced::run(&ActionAtomic::Rollback, &item),
        Ok(AtomicStep::Waiting)
    ));
    assert_eq!(read(&item.path), "old");
    assert_eq!(dir.entries(), [PATH]);
}

#[test]
fn a_creation_can_never_be_at_replaced() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("new"), Some("new"), Some("old"));

    assert!(refused(replaced::run(&ActionAtomic::Create, &item)));
    assert_eq!(dir.entries(), [PATH, BACKUP, TEMP]);
}

// ── Done ─────────────────────────────────────────────────────────────────────

#[test]
fn done_stays_done_without_touching_the_disk() {
    for action in [ActionAtomic::Create, ActionAtomic::Modify] {
        let dir = TempDir::new();
        let item = item(&dir, bytes("new"));
        lay_out(&item, Some("new"), Some("new"), Some("old"));

        assert!(matches!(done::run(&action, &item), Ok(AtomicStep::Done)));
        assert_eq!(dir.entries(), [PATH, BACKUP, TEMP]);
    }
}

#[test]
fn done_finish_removes_the_backup_and_the_temp_file_and_can_run_twice() {
    for backup in [Some("old"), None] {
        let dir = TempDir::new();
        let item = item(&dir, bytes("new"));
        lay_out(&item, Some("new"), Some("new"), backup);

        for _ in 0..2 {
            assert!(matches!(
                done::run(&ActionAtomic::Finish, &item),
                Ok(AtomicStep::Done)
            ));
            assert_eq!(read(&item.path), "new");
            assert_eq!(dir.entries(), [PATH]);
        }
    }
}

#[test]
fn done_rollback_puts_the_original_back_when_there_was_one() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("new"), Some("new"), Some("old"));

    assert!(matches!(
        done::run(&ActionAtomic::Rollback, &item),
        Ok(AtomicStep::Waiting)
    ));
    assert_eq!(read(&item.path), "old");
    assert_eq!(dir.entries(), [PATH]);
}

#[test]
fn done_rollback_removes_the_file_when_nothing_was_there_before() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("new"), Some("new"), None);

    assert!(matches!(
        done::run(&ActionAtomic::Rollback, &item),
        Ok(AtomicStep::Waiting)
    ));
    assert!(dir.entries().is_empty());
}

// ── Cells that are refused, and a rollback run again ─────────────────────────

#[test]
fn finish_is_refused_before_done() {
    let steps: [fn(&ActionAtomic, &ItemsPath) -> io::Result<AtomicStep>; 4] =
        [waiting::run, prepared::run, placed::run, replaced::run];
    for run in steps {
        let dir = TempDir::new();
        let item = item(&dir, bytes("new"));
        lay_out(&item, Some("new"), Some("new"), Some("old"));

        assert!(refused(run(&ActionAtomic::Finish, &item)));
        assert_eq!(dir.entries(), [PATH, BACKUP, TEMP]);
    }
}

// A restore cut right before its last line leaves the original at `path` and
// the backup still there: running it again must not lose the original.
#[test]
fn a_restore_run_again_keeps_the_original() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    lay_out(&item, Some("old"), None, Some("old"));

    assert!(matches!(rollback::restore(&item), Ok(AtomicStep::Waiting)));
    assert_eq!(read(&item.path), "old");
    assert_eq!(dir.entries(), [PATH]);
}

// ── Odd layouts ──────────────────────────────────────────────────────────────

// `sub/../a.txt` is `a.txt` again: the second entry sees the first one's file
// as its original. Undone in the order of insertion, that content would be put
// back as if it had been there before the batch.
#[cfg(unix)]
#[test]
fn two_spellings_of_one_path_in_a_modify_batch_are_undone_to_nothing() {
    let dir = TempDir::new();
    fs::create_dir(dir.join("sub")).unwrap();
    fs::create_dir(dir.join("blocked")).unwrap();

    let outcome = AtomicBuilder::modify()
        .add(dir.join("a.txt"), bytes("first"), false)
        .unwrap()
        .add(dir.join("sub/../a.txt"), bytes("second"), false)
        .unwrap()
        .add(dir.join("blocked"), bytes("new"), false)
        .unwrap()
        .start();

    assert!(outcome.is_err());
    assert_eq!(dir.entries(), ["blocked", "sub"]);
}

#[cfg(unix)]
#[test]
fn a_symlink_at_the_path_is_replaced_and_its_target_left_alone() {
    let dir = TempDir::new();
    write(&dir.join("target.txt"), "target");
    std::os::unix::fs::symlink(dir.join("target.txt"), dir.join("link.txt")).unwrap();

    AtomicBuilder::modify()
        .add(dir.join("link.txt"), bytes("new"), false)
        .unwrap()
        .start()
        .unwrap();

    let link = fs::symlink_metadata(dir.join("link.txt")).unwrap();
    assert!(!link.file_type().is_symlink());
    assert_eq!(read(&dir.join("link.txt")), "new");
    assert_eq!(read(&dir.join("target.txt")), "target");
    assert_eq!(dir.entries(), ["link.txt", "target.txt"]);
}

#[cfg(unix)]
#[test]
fn a_symlink_at_the_path_comes_back_after_a_rollback() {
    let dir = TempDir::new();
    write(&dir.join("target.txt"), "target");
    std::os::unix::fs::symlink(dir.join("target.txt"), dir.join("link.txt")).unwrap();
    fs::create_dir(dir.join("blocked")).unwrap();

    let outcome = AtomicBuilder::modify()
        .add(dir.join("link.txt"), bytes("new"), false)
        .unwrap()
        .add(dir.join("blocked"), bytes("new"), false)
        .unwrap()
        .start();

    assert!(outcome.is_err());
    let link = fs::symlink_metadata(dir.join("link.txt")).unwrap();
    assert!(link.file_type().is_symlink());
    assert_eq!(read(&dir.join("link.txt")), "target");
    assert_eq!(read(&dir.join("target.txt")), "target");
    assert_eq!(dir.entries(), ["blocked", "link.txt", "target.txt"]);
}

#[cfg(unix)]
#[test]
fn a_read_only_original_comes_back_read_only() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new();
    write(&dir.join("a.txt"), "before");
    fs::set_permissions(dir.join("a.txt"), fs::Permissions::from_mode(0o444)).unwrap();
    fs::create_dir(dir.join("blocked")).unwrap();

    let outcome = AtomicBuilder::modify()
        .add(dir.join("a.txt"), bytes("after"), false)
        .unwrap()
        .add(dir.join("blocked"), bytes("new"), false)
        .unwrap()
        .start();

    assert!(outcome.is_err());
    assert_eq!(read(&dir.join("a.txt")), "before");
    assert_eq!(mode(&dir.join("a.txt")), 0o444);
    assert_eq!(dir.entries(), ["a.txt", "blocked"]);
}

#[cfg(unix)]
#[test]
fn a_private_file_replacing_an_ordinary_one_ends_up_private() {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new();
    write(&dir.join(".env"), "old");
    fs::set_permissions(dir.join(".env"), fs::Permissions::from_mode(0o644)).unwrap();

    AtomicBuilder::modify()
        .add(dir.join(".env"), bytes("new"), true)
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(read(&dir.join(".env")), "new");
    assert_eq!(mode(&dir.join(".env")), 0o600);
    assert_eq!(dir.entries(), [".env"]);
}

// A file sits where a directory is needed: the directories made for the other
// files of the batch are taken away again.
#[test]
fn a_batch_that_cannot_create_its_directories_changes_nothing() {
    let dir = TempDir::new();
    write(&dir.join("blocker"), "a file");

    let outcome = AtomicBuilder::create()
        .add(dir.join("made/a.txt"), bytes("a"), false)
        .unwrap()
        .add(dir.join("blocker/sub/b.txt"), bytes("b"), false)
        .unwrap()
        .start();

    assert!(outcome.is_err());
    assert_eq!(read(&dir.join("blocker")), "a file");
    assert_eq!(dir.entries(), ["blocker"]);
}

#[test]
fn a_directory_squatting_the_temp_path_stops_the_batch() {
    let dir = TempDir::new();
    write(&dir.join(PATH), "mine");
    fs::create_dir(dir.join(TEMP)).unwrap();

    let outcome = AtomicBuilder::modify()
        .add(dir.join(PATH), bytes("new"), false)
        .unwrap()
        .start();

    assert!(outcome.is_err());
    assert_eq!(read(&dir.join(PATH)), "mine");
    assert_eq!(dir.entries(), [PATH, TEMP]);
}

#[test]
fn a_directory_squatting_the_backup_path_stops_the_batch() {
    let dir = TempDir::new();
    write(&dir.join(PATH), "mine");
    fs::create_dir(dir.join(BACKUP)).unwrap();

    let outcome = AtomicBuilder::modify()
        .add(dir.join(PATH), bytes("new"), false)
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join(PATH)), "mine");
    assert_eq!(dir.entries(), [PATH, BACKUP]);
}

#[test]
fn a_create_batch_run_twice_refuses_the_second_time() {
    let dir = TempDir::new();
    let batch = |text| {
        AtomicBuilder::create()
            .add(dir.join("a.txt"), bytes(text), false)
            .unwrap()
            .start()
    };

    batch("first").unwrap();
    assert!(already_exists(batch("second")));

    assert_eq!(read(&dir.join("a.txt")), "first");
    assert_eq!(dir.entries(), ["a.txt"]);
}

#[test]
fn a_modify_batch_can_run_again_and_again() {
    let dir = TempDir::new();
    for text in ["one", "two", "three"] {
        AtomicBuilder::modify()
            .add(dir.join("a.txt"), bytes(text), false)
            .unwrap()
            .start()
            .unwrap();
        assert_eq!(read(&dir.join("a.txt")), text);
        assert_eq!(dir.entries(), ["a.txt"]);
    }
}

#[test]
fn an_empty_file_under_an_odd_name_is_written_as_it_is() {
    let dir = TempDir::new();
    AtomicBuilder::create()
        .add(dir.join("dossier été/fichier à moi.txt"), bytes(""), false)
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(read(&dir.join("dossier été/fichier à moi.txt")), "");
    assert_eq!(
        dir.entries(),
        ["dossier été", "dossier été/fichier à moi.txt"]
    );
}

#[test]
fn a_large_create_batch_failing_on_its_last_file_leaves_nothing() {
    let dir = TempDir::new();
    write(&dir.join("taken.txt"), "mine");
    let mut batch = AtomicBuilder::create();
    for index in 0..30 {
        let name = format!("d{index}/sub/f{index}.txt");
        batch = batch.add(dir.join(&name), bytes("x"), false).unwrap();
    }

    let outcome = batch
        .add(dir.join("taken.txt"), bytes("theirs"), false)
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join("taken.txt")), "mine");
    assert_eq!(dir.entries(), ["taken.txt"]);
}

#[test]
fn a_failure_names_the_file_it_happened_on() {
    let dir = TempDir::new();
    write(&dir.join("taken.txt"), "mine");

    let error = AtomicBuilder::create()
        .add(dir.join("new.txt"), bytes("new"), false)
        .unwrap()
        .add(dir.join("taken.txt"), bytes("theirs"), false)
        .unwrap()
        .start()
        .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    let message = error.to_string();
    assert!(message.contains("taken.txt"), "{message}");
    assert!(!message.contains("new.txt"), "{message}");
}

// ── An upload, as the form fields would hand it over ─────────────────────────
//
// The media root holds a `.staging-*` folder with the received files, already
// renamed. The field has checked them; the batch only has to publish them.

/// A media root with `names` received in its staging folder, content = name.
fn staged(dir: &TempDir, names: &[&str]) -> Vec<PathBuf> {
    fs::create_dir(dir.join(".staging-1")).unwrap();
    names
        .iter()
        .map(|name| {
            let path = dir.join(&format!(".staging-1/{name}"));
            write(&path, name);
            path
        })
        .collect()
}

#[test]
fn an_upload_is_published_under_a_folder_made_for_it() {
    let dir = TempDir::new();
    let files = staged(&dir, &["photo-9c41.jpg"]);

    AtomicBuilder::create()
        .add(
            dir.join("avatars/photo-9c41.jpg"),
            SourceItems::File(files[0].clone()),
            false,
        )
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(read(&dir.join("avatars/photo-9c41.jpg")), "photo-9c41.jpg");
    assert_eq!(
        dir.entries(),
        [
            ".staging-1",
            ".staging-1/photo-9c41.jpg",
            "avatars",
            "avatars/photo-9c41.jpg"
        ]
    );
}

// Same inode on both names: the content was linked, not read and written again.
#[cfg(unix)]
#[test]
fn an_upload_is_linked_not_copied() {
    use std::os::unix::fs::MetadataExt;
    let dir = TempDir::new();
    let files = staged(&dir, &["video.mp4"]);

    AtomicBuilder::create()
        .add(
            dir.join("video.mp4"),
            SourceItems::File(files[0].clone()),
            false,
        )
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(
        fs::metadata(dir.join("video.mp4")).unwrap().ino(),
        fs::metadata(&files[0]).unwrap().ino()
    );
}

// What the request does once the form is saved: the staging folder goes, the
// published file stays whole.
#[test]
fn a_published_upload_outlives_its_staging_folder() {
    let dir = TempDir::new();
    let files = staged(&dir, &["photo.jpg"]);

    AtomicBuilder::create()
        .add(
            dir.join("avatars/photo.jpg"),
            SourceItems::File(files[0].clone()),
            false,
        )
        .unwrap()
        .start()
        .unwrap();
    fs::remove_dir_all(dir.join(".staging-1")).unwrap();

    assert_eq!(read(&dir.join("avatars/photo.jpg")), "photo.jpg");
    assert_eq!(dir.entries(), ["avatars", "avatars/photo.jpg"]);
}

#[test]
fn several_uploads_of_one_field_are_published_together() {
    let dir = TempDir::new();
    let files = staged(&dir, &["a.jpg", "b.jpg", "c.jpg"]);
    let mut batch = AtomicBuilder::create();
    for (file, name) in files.iter().zip(["a.jpg", "b.jpg", "c.jpg"]) {
        batch = batch
            .add(
                dir.join(&format!("gallery/{name}")),
                SourceItems::File(file.clone()),
                false,
            )
            .unwrap();
    }

    batch.start().unwrap();

    for name in ["a.jpg", "b.jpg", "c.jpg"] {
        assert_eq!(read(&dir.join(&format!("gallery/{name}"))), name);
    }
}

// One name of three is already taken: none is published, the folder made for
// them is gone, and the three received files are still in staging.
#[test]
fn several_uploads_are_published_all_or_none() {
    let dir = TempDir::new();
    let files = staged(&dir, &["a.jpg", "b.jpg", "c.jpg"]);
    fs::create_dir(dir.join("kept")).unwrap();
    write(&dir.join("kept/b.jpg"), "mine");

    let outcome = AtomicBuilder::create()
        .add(
            dir.join("gallery/a.jpg"),
            SourceItems::File(files[0].clone()),
            false,
        )
        .unwrap()
        .add(
            dir.join("kept/b.jpg"),
            SourceItems::File(files[1].clone()),
            false,
        )
        .unwrap()
        .add(
            dir.join("gallery/c.jpg"),
            SourceItems::File(files[2].clone()),
            false,
        )
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join("kept/b.jpg")), "mine");
    assert_eq!(
        dir.entries(),
        [
            ".staging-1",
            ".staging-1/a.jpg",
            ".staging-1/b.jpg",
            ".staging-1/c.jpg",
            "kept",
            "kept/b.jpg"
        ]
    );
}

// The request's staging folder was swept before the save reached the files.
#[test]
fn an_upload_gone_from_staging_publishes_nothing() {
    let dir = TempDir::new();
    let files = staged(&dir, &["a.jpg", "b.jpg"]);
    fs::remove_file(&files[1]).unwrap();

    let outcome = AtomicBuilder::create()
        .add(
            dir.join("gallery/a.jpg"),
            SourceItems::File(files[0].clone()),
            false,
        )
        .unwrap()
        .add(
            dir.join("gallery/b.jpg"),
            SourceItems::File(files[1].clone()),
            false,
        )
        .unwrap()
        .start();

    let error = outcome.unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert!(error.to_string().contains("b.jpg"), "{error}");
    assert_eq!(dir.entries(), [".staging-1", ".staging-1/a.jpg"]);
}

// The files are published, then the database refuses the commit: taking each
// one back is a plain removal, since a creation never had anything to restore.
#[test]
fn published_uploads_are_taken_back_when_the_commit_fails() {
    let dir = TempDir::new();
    let files = staged(&dir, &["a.jpg", "b.jpg"]);
    let published = [dir.join("gallery/a.jpg"), dir.join("gallery/b.jpg")];
    AtomicBuilder::create()
        .add(
            published[0].clone(),
            SourceItems::File(files[0].clone()),
            false,
        )
        .unwrap()
        .add(
            published[1].clone(),
            SourceItems::File(files[1].clone()),
            false,
        )
        .unwrap()
        .start()
        .unwrap();

    for path in &published {
        disk::remove(path).unwrap();
        disk::remove(path).unwrap();
    }

    assert_eq!(read(&files[0]), "a.jpg");
    assert_eq!(read(&files[1]), "b.jpg");
    assert_eq!(
        dir.entries(),
        [
            ".staging-1",
            ".staging-1/a.jpg",
            ".staging-1/b.jpg",
            "gallery"
        ]
    );
}

// A profile picture replaced: the new one is published by the batch, the old
// one is removed afterwards by whoever decided it was no longer wanted.
#[test]
fn the_file_an_upload_replaces_is_removed_on_request_only() {
    let dir = TempDir::new();
    let files = staged(&dir, &["photo-new.jpg"]);
    fs::create_dir(dir.join("avatars")).unwrap();
    write(&dir.join("avatars/photo-old.jpg"), "old");
    write(&dir.join("avatars/background.avif"), "background");

    AtomicBuilder::create()
        .add(
            dir.join("avatars/photo-new.jpg"),
            SourceItems::File(files[0].clone()),
            false,
        )
        .unwrap()
        .start()
        .unwrap();
    assert_eq!(read(&dir.join("avatars/photo-old.jpg")), "old");

    disk::remove(&dir.join("avatars/photo-old.jpg")).unwrap();

    assert_eq!(read(&dir.join("avatars/photo-new.jpg")), "photo-new.jpg");
    assert_eq!(read(&dir.join("avatars/background.avif")), "background");
    assert_eq!(
        dir.entries(),
        [
            ".staging-1",
            ".staging-1/photo-new.jpg",
            "avatars",
            "avatars/background.avif",
            "avatars/photo-new.jpg"
        ]
    );
}

// The same upload published twice under one name: the second attempt is
// refused and the first file is left as it was.
#[test]
fn publishing_the_same_upload_twice_is_refused() {
    let dir = TempDir::new();
    let files = staged(&dir, &["photo.jpg"]);
    let publish = || {
        AtomicBuilder::create()
            .add(
                dir.join("avatars/photo.jpg"),
                SourceItems::File(files[0].clone()),
                false,
            )
            .unwrap()
            .start()
    };

    publish().unwrap();
    assert!(already_exists(publish()));

    assert_eq!(read(&dir.join("avatars/photo.jpg")), "photo.jpg");
}

// ── After a power cut ────────────────────────────────────────────────────────
//
// A cut can't be triggered from a test, but what it leaves on disk can be laid
// out by hand: each test starts from such a state and runs a batch again.

const MB: usize = 1024 * 1024;

// 10 MB of a 20 MB temp file were written when the power went. `path` was not
// reached yet. Without a journal the batch can't tell that file is its own:
// the next one is refused, and nothing is removed.
#[test]
fn after_a_cut_in_a_half_written_temp_file_the_next_batch_is_refused() {
    for creating in [true, false] {
        let dir = TempDir::new();
        let content = vec![7u8; 20 * MB];
        fs::write(dir.join(TEMP), &content[..10 * MB]).unwrap();
        if !creating {
            write(&dir.join(PATH), "original");
        }
        let before = dir.entries();

        let batch = if creating {
            AtomicBuilder::create()
        } else {
            AtomicBuilder::modify()
        };
        let error = batch
            .add(dir.join(PATH), SourceItems::Bytes(content.clone()), false)
            .unwrap()
            .start()
            .unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
        assert!(error.to_string().contains(TEMP), "{error}");
        assert_eq!(fs::read(dir.join(TEMP)).unwrap().len(), 10 * MB);
        assert_eq!(dir.entries(), before);
    }
}

// The cut came while the temp file was being written: the file itself was
// never touched, whatever the size.
#[test]
fn after_a_cut_in_a_half_written_temp_file_the_original_is_whole() {
    let dir = TempDir::new();
    let original = vec![1u8; 20 * MB];
    fs::write(dir.join(PATH), &original).unwrap();
    fs::write(dir.join(TEMP), vec![2u8; 10 * MB]).unwrap();

    assert!(fs::read(dir.join(PATH)).unwrap() == original);
    assert_eq!(dir.entries(), [PATH, TEMP]);
}

// Three files to create, the cut after the first was placed: the first is on
// disk with its temp file, the second has its temp file only. Without a
// journal a creation is not resumed: the next batch is refused and leaves all
// of it as it found it.
#[test]
fn after_a_cut_between_two_placements_a_creation_is_not_resumed() {
    let dir = TempDir::new();
    write(&dir.join("a.bin"), "a");
    fs::hard_link(dir.join("a.bin"), dir.join("a.bin.runique-tmp")).unwrap();
    write(&dir.join("b.bin.runique-tmp"), "b");

    let error = AtomicBuilder::create()
        .add(dir.join("a.bin"), bytes("a"), false)
        .unwrap()
        .add(dir.join("b.bin"), bytes("b"), false)
        .unwrap()
        .add(dir.join("c.bin"), bytes("c"), false)
        .unwrap()
        .start()
        .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert!(error.to_string().contains("a.bin"), "{error}");
    assert_eq!(read(&dir.join("a.bin")), "a");
    assert_eq!(
        dir.entries(),
        ["a.bin", "a.bin.runique-tmp", "b.bin.runique-tmp"]
    );
}

// A replacement cut at each of its three moments: the backup just made, the
// original taken away, the new content placed. Each time a backup is left, and
// each time the next batch is refused without touching anything.
#[test]
fn after_a_cut_in_a_replacement_the_next_batch_is_refused() {
    for at_path in [Some("original"), None, Some("new")] {
        let dir = TempDir::new();
        let cut = item(&dir, bytes("new"));
        lay_out(&cut, at_path, Some("new"), Some("original"));
        let before = dir.entries();

        let outcome = AtomicBuilder::modify()
            .add(dir.join(PATH), bytes("newer"), false)
            .unwrap()
            .start();

        assert!(already_exists(outcome), "{at_path:?}");
        assert_eq!(read(&dir.join(BACKUP)), "original", "{at_path:?}");
        assert_eq!(read(&dir.join(TEMP)), "new", "{at_path:?}");
        assert_eq!(dir.join(PATH).exists(), at_path.is_some());
        if let Some(text) = at_path {
            assert_eq!(read(&dir.join(PATH)), text);
        }
        assert_eq!(dir.entries(), before, "{at_path:?}");
    }
}

// The worst moment: the original is in the backup and nothing is at `path`.
// Nothing is lost: put back by hand, and the temp file cleared, the file is
// whole and the batch goes through.
#[test]
fn after_a_cut_the_original_can_be_put_back_from_the_backup_by_hand() {
    let dir = TempDir::new();
    let cut = item(&dir, bytes("new"));
    lay_out(&cut, None, Some("new"), Some("original"));

    fs::rename(dir.join(BACKUP), dir.join(PATH)).unwrap();
    fs::remove_file(dir.join(TEMP)).unwrap();
    assert_eq!(read(&dir.join(PATH)), "original");

    AtomicBuilder::modify()
        .add(dir.join(PATH), bytes("newer"), false)
        .unwrap()
        .start()
        .unwrap();
    assert_eq!(read(&dir.join(PATH)), "newer");
    assert_eq!(dir.entries(), [PATH]);
}

// The cut came during the final cleanup: the backup is gone, a temp file is
// left. Without a journal, that temp file stops the next batch too.
#[test]
fn after_a_cut_in_the_cleanup_the_leftover_temp_file_stops_the_next_batch() {
    let dir = TempDir::new();
    let cut = item(&dir, bytes("new"));
    lay_out(&cut, Some("new"), Some("new"), None);

    let outcome = AtomicBuilder::modify()
        .add(dir.join(PATH), bytes("newer"), false)
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join(PATH)), "new");
    assert_eq!(dir.entries(), [PATH, TEMP]);
}

// On a disk without hard links the file is copied, and a cut can leave half of
// it at `path`: 10 MB of 20. A creation refuses to touch it; a modification
// takes it for the original and replaces it with the whole file.
#[test]
fn after_a_cut_in_a_copy_the_half_written_file_is_never_taken_for_a_whole_one() {
    let content = vec![9u8; 20 * MB];

    let dir = TempDir::new();
    fs::write(dir.join(PATH), &content[..10 * MB]).unwrap();
    let outcome = AtomicBuilder::create()
        .add(dir.join(PATH), SourceItems::Bytes(content.clone()), false)
        .unwrap()
        .start();
    assert!(already_exists(outcome));
    assert_eq!(fs::read(dir.join(PATH)).unwrap().len(), 10 * MB);
    assert_eq!(dir.entries(), [PATH]);

    AtomicBuilder::modify()
        .add(dir.join(PATH), SourceItems::Bytes(content.clone()), false)
        .unwrap()
        .start()
        .unwrap();
    assert!(fs::read(dir.join(PATH)).unwrap() == content);
    assert_eq!(dir.entries(), [PATH]);
}

// ── A path without a folder ──────────────────────────────────────────────────

// No parent to create, and the folder to sync is the current one.
#[test]
fn a_file_without_a_parent_folder_is_written_in_the_current_one() {
    struct Cleanup(String);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            for suffix in ["", ".runique-tmp", ".runique-bak"] {
                let _ = fs::remove_file(format!("{}{suffix}", self.0));
            }
        }
    }
    let name = Cleanup(format!("rq_atomic_bare_{}.txt", uuid::Uuid::new_v4()));

    AtomicBuilder::create()
        .add(PathBuf::from(&name.0), bytes("bare"), false)
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(read(Path::new(&name.0)), "bare");
    assert!(!Path::new(&format!("{}.runique-tmp", name.0)).exists());
}

// ── The journal ──────────────────────────────────────────────────────────────

/// Leaves in `dir` the journal of a batch cut short: each file by its name
/// and whether it was there before, then the folders the batch had created.
pub(crate) fn cut_journal(dir: &Path, placed: bool, files: &[(&str, bool)], dirs: &[&str]) {
    let journal = Journal {
        placed,
        files: files
            .iter()
            .map(|(name, existed)| JournalFile {
                path: PathBuf::from(name),
                existed: *existed,
            })
            .collect(),
        dirs: dirs.iter().map(PathBuf::from).collect(),
    };
    journal::write(dir, &journal).unwrap();
}

#[test]
fn a_batch_with_a_journal_leaves_none_once_it_is_done() {
    let dir = TempDir::new();
    write(&dir.join("old.txt"), "before");

    AtomicBuilder::modify()
        .journal(dir.path())
        .add(dir.join("old.txt"), bytes("after"), false)
        .unwrap()
        .add(dir.join("sub/new.txt"), bytes("new"), false)
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(read(&dir.join("old.txt")), "after");
    assert_eq!(dir.entries(), ["old.txt", "sub", "sub/new.txt"]);
    assert!(!recover(dir.path()).unwrap());
}

// The journal's folder does not exist yet: it is created with the batch, and
// goes with it when the batch is undone. (A path that climbs with `..` is
// refused by the journal, once the folders are made.)
#[test]
fn a_batch_with_a_journal_leaves_none_once_it_is_undone() {
    let dir = TempDir::new();

    let outcome = AtomicBuilder::create()
        .journal(dir.join("project"))
        .add(
            dir.join("project/src/main.rs"),
            bytes("fn main() {}"),
            false,
        )
        .unwrap()
        .add(
            dir.join("project/src/../src/main.rs"),
            bytes("fn main() {}"),
            false,
        )
        .unwrap()
        .start();

    assert!(outcome.is_err());
    assert!(dir.entries().is_empty());
}

// The folders above the journal's own had to be made too: they are undone
// with the batch while it runs, and a batch that succeeds keeps them.
#[test]
fn a_journal_folder_several_levels_down_is_created_with_its_parents() {
    let dir = TempDir::new();

    AtomicBuilder::create()
        .journal(dir.join("apps/blog/migration"))
        .add(
            dir.join("apps/blog/migration/Cargo.toml"),
            bytes("x"),
            false,
        )
        .unwrap()
        .start()
        .unwrap();
    assert_eq!(
        dir.entries(),
        [
            "apps",
            "apps/blog",
            "apps/blog/migration",
            "apps/blog/migration/Cargo.toml"
        ]
    );

    let outcome = AtomicBuilder::create()
        .journal(dir.join("other/deep/migration"))
        .add(dir.join("other/deep/migration/a.txt"), bytes("a"), false)
        .unwrap()
        .add(
            dir.join("other/deep/migration/sub/../a.txt"),
            bytes("a"),
            false,
        )
        .unwrap()
        .start();
    assert!(outcome.is_err());
    assert!(!dir.join("other").exists());
}

#[test]
fn a_journal_in_a_folder_the_batch_creates_is_removed_from_it() {
    let dir = TempDir::new();

    AtomicBuilder::create()
        .journal(dir.join("project"))
        .add(
            dir.join("project/src/main.rs"),
            bytes("fn main() {}"),
            false,
        )
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(
        dir.entries(),
        ["project", "project/src", "project/src/main.rs"]
    );
}

// A path outside the journal's folder could not be found again from another
// working directory: the batch is refused before it touches anything.
#[test]
fn a_file_outside_the_journal_s_folder_is_refused() {
    let dir = TempDir::new();

    let error = AtomicBuilder::create()
        .journal(dir.join("inner"))
        .add(dir.join("outer.txt"), bytes("x"), false)
        .unwrap()
        .start()
        .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    assert!(dir.entries().is_empty());
}

#[test]
fn an_empty_batch_writes_no_journal() {
    let dir = TempDir::new();
    AtomicBuilder::modify().journal(dir.path()).start().unwrap();
    assert!(dir.entries().is_empty());
}

#[test]
fn without_a_journal_there_is_nothing_to_repair() {
    let dir = TempDir::new();
    write(&dir.join(PATH), "mine");

    assert!(!recover(dir.path()).unwrap());
    assert!(!recover(&dir.join("missing")).unwrap());

    assert_eq!(dir.entries(), [PATH]);
}

// A file that was there before the batch, cut at each moment of its
// replacement: not reached yet, backed up, taken away, replaced.
#[test]
fn a_cut_replacement_gives_the_original_back() {
    for (at_path, backup) in [
        (Some("original"), None),
        (Some("original"), Some("original")),
        (None, Some("original")),
        (Some("new"), Some("original")),
    ] {
        let dir = TempDir::new();
        let cut = item(&dir, bytes("new"));
        lay_out(&cut, at_path, Some("new"), backup);
        cut_journal(dir.path(), false, &[(PATH, true)], &[]);

        assert!(recover(dir.path()).unwrap());

        assert_eq!(read(&dir.join(PATH)), "original", "{at_path:?} {backup:?}");
        assert_eq!(dir.entries(), [PATH], "{at_path:?} {backup:?}");
    }
}

// A file that was not there before: whatever the batch left is taken away,
// and the folders it had made with it.
#[test]
fn a_cut_creation_leaves_nothing() {
    for (at_path, temp) in [(None, None), (None, Some("ne")), (Some("new"), Some("new"))] {
        let dir = TempDir::new();
        fs::create_dir_all(dir.join("sub/deep")).unwrap();
        for (name, text) in [
            ("sub/deep/file.txt", at_path),
            ("sub/deep/file.txt.runique-tmp", temp),
        ] {
            if let Some(text) = text {
                write(&dir.join(name), text);
            }
        }
        cut_journal(
            dir.path(),
            false,
            &[("sub/deep/file.txt", false)],
            &["sub", "sub/deep"],
        );

        assert!(recover(dir.path()).unwrap());

        assert!(dir.entries().is_empty(), "{at_path:?} {temp:?}");
    }
}

// A creation that met a file of the user's and was cut before it could fail:
// that file was there before, the repair leaves it alone.
#[test]
fn a_file_that_was_there_before_a_cut_creation_is_left_alone() {
    let dir = TempDir::new();
    let cut = item(&dir, bytes("new"));
    lay_out(&cut, Some("mine"), Some("new"), None);
    cut_journal(dir.path(), false, &[(PATH, true)], &[]);

    assert!(recover(dir.path()).unwrap());

    assert_eq!(read(&dir.join(PATH)), "mine");
    assert_eq!(dir.entries(), [PATH]);
}

// Every file was in place: the new content stays, only what the cleanup had
// not removed yet goes.
#[test]
fn a_cut_cleanup_is_finished_and_keeps_the_new_content() {
    for (temp, backup) in [
        (Some("new"), Some("original")),
        (Some("new"), None),
        (None, None),
    ] {
        let dir = TempDir::new();
        let cut = item(&dir, bytes("new"));
        lay_out(&cut, Some("new"), temp, backup);
        cut_journal(dir.path(), true, &[(PATH, true)], &[]);

        assert!(recover(dir.path()).unwrap());

        assert_eq!(read(&dir.join(PATH)), "new");
        assert_eq!(dir.entries(), [PATH], "{temp:?} {backup:?}");
    }
}

// A directory where the placed file is expected: the repair can't remove it.
// The journal stays, and the repair goes through once the way is clear.
#[test]
fn a_repair_that_fails_keeps_the_journal_for_the_next_start() {
    let dir = TempDir::new();
    fs::create_dir(dir.join(PATH)).unwrap();
    cut_journal(dir.path(), false, &[(PATH, false)], &[]);

    assert!(recover(dir.path()).is_err());
    assert!(dir.join(".runique-journal").exists());

    fs::remove_dir(dir.join(PATH)).unwrap();
    assert!(recover(dir.path()).unwrap());
    assert!(dir.entries().is_empty());
}

#[test]
fn a_journal_that_cannot_be_read_stops_everything_and_touches_nothing() {
    let dir = TempDir::new();
    write(&dir.join(PATH), "mine");
    write(&dir.join(".runique-journal"), "not a journal");

    let error = recover(dir.path()).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(error.to_string().contains(".runique-journal"), "{error}");
    assert_eq!(read(&dir.join(PATH)), "mine");
    assert_eq!(dir.entries(), [".runique-journal", PATH]);
}

// A project moved since the cut: the journal names its files from its own
// folder, so the repair finds them where the folder now is.
#[test]
fn a_journal_is_repaired_wherever_its_folder_is() {
    let dir = TempDir::new();
    fs::create_dir(dir.join("here")).unwrap();
    write(&dir.join("here/file.txt"), "new");
    write(&dir.join("here/file.txt.runique-tmp"), "new");
    write(&dir.join("here/file.txt.runique-bak"), "original");
    cut_journal(&dir.join("here"), false, &[(PATH, true)], &[]);

    fs::rename(dir.join("here"), dir.join("there")).unwrap();
    assert!(recover(&dir.join("there")).unwrap());

    assert_eq!(read(&dir.join("there/file.txt")), "original");
    assert_eq!(dir.entries(), ["there", "there/file.txt"]);
}

// ── What the batch refuses to take, and what a repair refuses to touch ───────

// The names the batch works under: a file of the batch carrying one would be
// taken for the temp file or the backup of another, and removed with it.
#[test]
fn a_file_under_a_name_the_batch_keeps_for_itself_is_refused() {
    let dir = TempDir::new();
    for name in [
        "a.txt.runique-tmp",
        "a.txt.runique-bak",
        ".runique-journal",
        ".runique-journal.runique-tmp",
        "sub/b.runique-bak",
    ] {
        let outcome = AtomicBuilder::modify().add(dir.join(name), bytes("x"), false);
        assert!(
            matches!(outcome, Err(error) if error.kind() == io::ErrorKind::InvalidInput),
            "{name}"
        );
    }
    assert!(dir.entries().is_empty());
}

#[test]
fn a_path_that_names_no_file_is_refused() {
    let dir = TempDir::new();
    for path in [PathBuf::new(), PathBuf::from("/"), dir.join("..")] {
        let outcome = AtomicBuilder::create().add(path.clone(), bytes("x"), false);
        assert!(
            matches!(outcome, Err(error) if error.kind() == io::ErrorKind::InvalidInput),
            "{path:?}"
        );
    }
}

// Close to a kept name, but not one: these are ordinary files.
#[test]
fn a_name_that_only_looks_like_a_kept_one_is_an_ordinary_file() {
    let dir = TempDir::new();
    let names = [
        "runique-tmp",
        "a.runique-tmp.txt",
        "a.runique-bakery",
        "runique-journal",
    ];
    let mut batch = AtomicBuilder::create();
    for name in names {
        batch = batch.add(dir.join(name), bytes(name), false).unwrap();
    }

    batch.start().unwrap();

    for name in names {
        assert_eq!(read(&dir.join(name)), name);
    }
}

// Nothing was there before the batch, and what is there now is not what the
// batch was placing: someone put a file there since the cut. The repair stops
// and touches nothing; once that file is out of the way, it goes through.
#[test]
fn a_file_the_batch_did_not_place_stops_the_repair() {
    let dir = TempDir::new();
    let cut = item(&dir, bytes("new"));
    lay_out(&cut, Some("theirs"), Some("new"), None);
    cut_journal(dir.path(), false, &[(PATH, false)], &[]);
    let before = dir.entries();

    let error = recover(dir.path()).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(error.to_string().contains(PATH), "{error}");
    assert_eq!(read(&dir.join(PATH)), "theirs");
    assert_eq!(dir.entries(), before);

    fs::rename(dir.join(PATH), dir.join("theirs.txt")).unwrap();
    assert!(recover(dir.path()).unwrap());
    assert_eq!(dir.entries(), ["theirs.txt"]);
}

// The file is neither the new content nor what its backup holds: it was edited
// since the cut. Putting the backup back would lose the edit, dropping the
// backup would lose the original: the repair stops and touches nothing.
#[test]
fn a_file_edited_since_the_cut_stops_the_repair() {
    let dir = TempDir::new();
    write(&dir.join("first.txt"), "new first");
    write(&dir.join("first.txt.runique-tmp"), "new first");
    write(&dir.join("first.txt.runique-bak"), "original first");
    let cut = item(&dir, bytes("new"));
    lay_out(&cut, Some("edited by hand"), Some("new"), Some("original"));
    cut_journal(dir.path(), false, &[("first.txt", true), (PATH, true)], &[]);
    let before = dir.entries();

    let error = recover(dir.path()).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(error.to_string().contains(PATH), "{error}");
    assert_eq!(read(&dir.join(PATH)), "edited by hand");
    assert_eq!(read(&dir.join(BACKUP)), "original");
    assert_eq!(
        read(&dir.join("first.txt")),
        "new first",
        "not even the first"
    );
    assert_eq!(dir.entries(), before);
}

// A journal is a file on the disk like any other: one naming a path that
// climbs out of its folder, an absolute one, or one of the batch's own names
// is refused whole, and nothing it names is touched.
#[test]
fn a_journal_naming_a_path_outside_its_folder_is_refused() {
    let dir = TempDir::new();
    fs::create_dir(dir.join("project")).unwrap();
    write(&dir.join("victim.txt"), "theirs");
    write(&dir.join("victim.txt.runique-tmp"), "theirs");
    let absolute = dir.join("victim.txt");
    // The files a journal names, then the folders.
    type Named<'a> = (&'a [(&'a str, bool)], &'a [&'a str]);
    let cases: [Named; 5] = [
        (&[("../victim.txt", false)], &[]),
        (&[(absolute.to_str().unwrap(), false)], &[]),
        (&[("file.txt.runique-bak", false)], &[]),
        (&[("sub/../../victim.txt", false)], &[]),
        (&[("file.txt", false)], &[".."]),
    ];
    for (files, dirs) in cases {
        cut_journal(&dir.join("project"), false, files, dirs);

        let error = recover(&dir.join("project")).unwrap_err();

        assert_eq!(
            error.kind(),
            io::ErrorKind::InvalidData,
            "{files:?} {dirs:?}"
        );
        assert_eq!(read(&dir.join("victim.txt")), "theirs", "{files:?}");
        assert!(dir.join("victim.txt.runique-tmp").exists(), "{files:?}");
        assert!(dir.join("project").is_dir(), "{files:?} {dirs:?}");
    }
}

// A folder of the project was replaced by a symlink to somewhere else: the
// file the journal names is no longer in the project, and is left alone.
#[cfg(unix)]
#[test]
fn a_symlinked_folder_does_not_lead_a_repair_out_of_the_project() {
    let dir = TempDir::new();
    fs::create_dir(dir.join("project")).unwrap();
    fs::create_dir(dir.join("outside")).unwrap();
    write(&dir.join("outside/file.txt"), "theirs");
    write(&dir.join("outside/file.txt.runique-tmp"), "theirs");
    std::os::unix::fs::symlink(dir.join("outside"), dir.join("project/src")).unwrap();
    cut_journal(&dir.join("project"), false, &[("src/file.txt", false)], &[]);

    let error = recover(&dir.join("project")).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(read(&dir.join("outside/file.txt")), "theirs");
    assert!(dir.join("outside/file.txt.runique-tmp").exists());
}

// The batch itself, while it runs, only ever removes what it created: a file
// that was there, a folder that was there, a folder it made but someone else
// filled.
#[test]
fn a_batch_undone_removes_only_what_it_created() {
    let dir = TempDir::new();
    fs::create_dir(dir.join("kept")).unwrap();
    write(&dir.join("kept/theirs.txt"), "theirs");
    write(&dir.join("taken.txt"), "mine");

    let outcome = AtomicBuilder::create()
        .journal(dir.path())
        .add(dir.join("kept/new.txt"), bytes("new"), false)
        .unwrap()
        .add(dir.join("made/deep/new.txt"), bytes("new"), false)
        .unwrap()
        .add(dir.join("taken.txt"), bytes("theirs"), false)
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join("kept/theirs.txt")), "theirs");
    assert_eq!(read(&dir.join("taken.txt")), "mine");
    assert_eq!(dir.entries(), ["kept", "kept/theirs.txt", "taken.txt"]);
}

// ── Only the states a cut can leave ──────────────────────────────────────────

// A temp file is the batch's own name: one that is already there when the
// batch starts was not made by it, and is neither written over nor removed.
#[test]
fn waiting_refuses_to_start_over_a_leftover_temp_file() {
    for action in [ActionAtomic::Create, ActionAtomic::Modify] {
        let dir = TempDir::new();
        let item = item(&dir, bytes("new"));
        lay_out(&item, None, Some("theirs"), None);

        assert!(already_exists(waiting::run(&action, &item)));
        assert_eq!(read(&item.temp_path), "theirs");
        assert_eq!(dir.entries(), [TEMP]);
    }
}

// Someone made `x.runique-tmp` by hand, with no batch in progress: the batch
// on `x` is refused, and names the file that is in the way.
#[test]
fn a_temp_file_made_by_hand_is_never_removed() {
    let dir = TempDir::new();
    write(&dir.join("notes.txt"), "notes");
    write(&dir.join("notes.txt.runique-tmp"), "a file of the user's");

    let error = AtomicBuilder::modify()
        .add(dir.join("other.txt"), bytes("other"), false)
        .unwrap()
        .add(dir.join("notes.txt"), bytes("new notes"), false)
        .unwrap()
        .start()
        .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::AlreadyExists);
    assert!(
        error.to_string().contains("notes.txt.runique-tmp"),
        "{error}"
    );
    assert_eq!(read(&dir.join("notes.txt")), "notes");
    assert_eq!(
        read(&dir.join("notes.txt.runique-tmp")),
        "a file of the user's"
    );
    assert_eq!(dir.entries(), ["notes.txt", "notes.txt.runique-tmp"]);
}

// A file that was there before the batch is gone, and no backup holds it:
// not a state a cut leaves. Nothing is touched.
#[test]
fn an_original_that_is_gone_without_a_backup_stops_the_repair() {
    let dir = TempDir::new();
    let cut = item(&dir, bytes("new"));
    lay_out(&cut, None, Some("new"), None);
    cut_journal(dir.path(), false, &[(PATH, true)], &[]);
    let before = dir.entries();

    let error = recover(dir.path()).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(dir.entries(), before);
}

// A backup next to a file that did not exist before the batch: no batch makes
// one. Removing it could lose something, so the repair stops.
#[test]
fn a_backup_of_a_file_that_was_not_there_stops_the_repair() {
    let dir = TempDir::new();
    let cut = item(&dir, bytes("new"));
    lay_out(&cut, Some("new"), Some("new"), Some("whose?"));
    cut_journal(dir.path(), false, &[(PATH, false)], &[]);
    let before = dir.entries();

    assert!(recover(dir.path()).is_err());

    assert_eq!(read(&dir.join(BACKUP)), "whose?");
    assert_eq!(dir.entries(), before);
}

// A file was added in a folder the batch had created: that folder could not
// be taken away with the rest. The repair stops before it removes anything.
#[test]
fn a_file_added_in_a_folder_the_batch_created_stops_the_repair() {
    let dir = TempDir::new();
    fs::create_dir_all(dir.join("made/deep")).unwrap();
    write(&dir.join("made/deep/file.txt"), "new");
    write(&dir.join("made/deep/file.txt.runique-tmp"), "new");
    write(&dir.join("made/notes.md"), "a file of the user's");
    cut_journal(
        dir.path(),
        false,
        &[("made/deep/file.txt", false)],
        &["made", "made/deep"],
    );
    let before = dir.entries();

    let error = recover(dir.path()).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(error.to_string().contains("notes.md"), "{error}");
    assert_eq!(dir.entries(), before);

    fs::rename(dir.join("made/notes.md"), dir.join("notes.md")).unwrap();
    assert!(recover(dir.path()).unwrap());
    assert_eq!(dir.entries(), ["notes.md"]);
}

// Two files don't match: both are named, one per line, for the command to show.
#[test]
fn every_file_that_does_not_match_is_listed() {
    let dir = TempDir::new();
    for name in ["a.txt", "b.txt"] {
        write(&dir.join(name), "edited by hand");
        write(&dir.join(&format!("{name}.runique-tmp")), "new");
        write(&dir.join(&format!("{name}.runique-bak")), "original");
    }
    write(&dir.join("c.txt"), "new");
    write(&dir.join("c.txt.runique-tmp"), "new");
    write(&dir.join("c.txt.runique-bak"), "original");
    cut_journal(
        dir.path(),
        false,
        &[("a.txt", true), ("b.txt", true), ("c.txt", true)],
        &[],
    );

    let message = recover(dir.path()).unwrap_err().to_string();

    assert_eq!(message.lines().count(), 2, "{message}");
    assert!(
        message.contains("a.txt") && message.contains("b.txt"),
        "{message}"
    );
    assert!(!message.contains("c.txt"), "{message}");
    assert_eq!(read(&dir.join("c.txt")), "new", "nothing was touched");
}

// A file the journal does not know, next to the batch's: not in a folder the
// batch created, it is no business of the repair.
#[test]
fn a_file_the_journal_does_not_know_is_left_where_it_is() {
    let dir = TempDir::new();
    let cut = item(&dir, bytes("new"));
    lay_out(&cut, Some("new"), Some("new"), Some("original"));
    write(&dir.join("by_hand.rs"), "// the developer's");
    cut_journal(dir.path(), false, &[(PATH, true)], &[]);

    assert!(recover(dir.path()).unwrap());

    assert_eq!(read(&dir.join(PATH)), "original");
    assert_eq!(read(&dir.join("by_hand.rs")), "// the developer's");
    assert_eq!(dir.entries(), ["by_hand.rs", PATH]);
}

// What a command that only reads the folder asks: is a batch waiting here?
#[test]
fn a_folder_is_interrupted_while_its_journal_is_there() {
    let dir = TempDir::new();
    assert!(!super::interrupted(dir.path()));
    assert!(!super::interrupted(&dir.join("missing")));

    let cut = item(&dir, bytes("new"));
    lay_out(&cut, Some("new"), Some("new"), Some("original"));
    cut_journal(dir.path(), false, &[(PATH, true)], &[]);
    assert!(super::interrupted(dir.path()));

    assert!(recover(dir.path()).unwrap());
    assert!(!super::interrupted(dir.path()));
}

// ── Survivors of the mutation run ────────────────────────────────────────────

/// A file on another filesystem than the temp folder's (`/dev/shm` is kept in
/// memory), removed on drop: a hard link can't reach it, so placing it has to
/// fall back on a copy. `None` where both sit on the same filesystem, or where
/// there is no `/dev/shm`: the test then has nothing to check.
#[cfg(target_os = "linux")]
pub(crate) struct Elsewhere(pub(crate) PathBuf);

#[cfg(target_os = "linux")]
impl Elsewhere {
    pub(crate) fn new(content: &str) -> Option<Self> {
        use std::os::unix::fs::MetadataExt;
        let shared = Path::new("/dev/shm");
        let here = fs::metadata(std::env::temp_dir()).ok()?.dev();
        if fs::metadata(shared).ok()?.dev() == here {
            return None;
        }
        let path = shared.join(format!("rq_atomic_{}", uuid::Uuid::new_v4()));
        fs::write(&path, content).ok()?;
        Some(Self(path))
    }
}

#[cfg(target_os = "linux")]
impl Drop for Elsewhere {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

// The whole batch on a source that can't be hard-linked: the content is
// copied, the source stays, and nothing of the batch's own is left.
#[cfg(target_os = "linux")]
#[test]
fn a_file_source_on_another_filesystem_is_copied() {
    let Some(source) = Elsewhere::new("upload") else {
        return;
    };
    let dir = TempDir::new();

    AtomicBuilder::create()
        .add(
            dir.join("made/final.bin"),
            SourceItems::File(source.0.clone()),
            false,
        )
        .unwrap()
        .start()
        .unwrap();

    assert_eq!(read(&dir.join("made/final.bin")), "upload");
    assert_eq!(read(&source.0), "upload");
    assert_eq!(dir.entries(), ["made", "made/final.bin"]);
}

// The same, with a file already at the destination: never written over,
// whether the content would have been linked or copied.
#[cfg(target_os = "linux")]
#[test]
fn a_file_source_on_another_filesystem_never_overwrites() {
    let Some(source) = Elsewhere::new("upload") else {
        return;
    };
    let dir = TempDir::new();
    write(&dir.join("final.bin"), "mine");

    let outcome = AtomicBuilder::create()
        .add(
            dir.join("final.bin"),
            SourceItems::File(source.0.clone()),
            false,
        )
        .unwrap()
        .start();

    assert!(already_exists(outcome));
    assert_eq!(read(&dir.join("final.bin")), "mine");
    assert_eq!(dir.entries(), ["final.bin"]);
}

// A directory at `path`: its backup can't be made. The error is the backup's
// own, not "a file is in the way" from placing the new content over it, and
// no half-made backup is left.
#[cfg(unix)]
#[test]
fn prepared_modify_reports_the_backup_that_failed() {
    let dir = TempDir::new();
    let item = item(&dir, bytes("new"));
    fs::create_dir(&item.path).unwrap();
    write(&item.temp_path, "new");

    let error = prepared::run(&ActionAtomic::Modify, &item).err().unwrap();

    assert_ne!(error.kind(), io::ErrorKind::AlreadyExists);
    assert!(item.path.is_dir());
    assert_eq!(dir.entries(), [PATH, TEMP]);
}

// An empty batch is the everyday case of a command that finds nothing to
// write: it makes no folder for a journal it has no use for.
#[test]
fn an_empty_batch_does_not_even_make_the_journal_s_folder() {
    let dir = TempDir::new();

    AtomicBuilder::create()
        .journal(dir.join("not/made"))
        .start()
        .unwrap();
    AtomicBuilder::modify()
        .journal(dir.join("not/made"))
        .start()
        .unwrap();

    assert!(dir.entries().is_empty());
}
