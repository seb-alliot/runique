//! Whole batches run through the builder's own entry points, then each cell of
//! the step × action table on a disk laid out by hand.
use super::{
    action::rollback,
    disk,
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
