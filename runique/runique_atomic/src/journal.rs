//! The batch's journal: what has to be on disk for a power cut to be repaired.
//! Written before the first file is touched, removed last. Found at the next
//! start, it tells which files to put back, or, when every file was already in
//! place, that only the cleanup is left to finish.
use crate::{
    builder::{backup_of, is_reserved, temp_of},
    disk,
    struct_atomic::{AtomicStep, ItemsPath, PathManager, SourceItems},
};
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

pub(super) const NAME: &str = ".runique-journal";
const TEMP_NAME: &str = ".runique-journal.runique-tmp";

/// A batch as a cut may find it. Paths are relative to the journal's folder:
/// the batch is repaired wherever the command is run from.
#[derive(Serialize, Deserialize)]
pub(super) struct Journal {
    /// Every file is in place and the cleanup may have removed backups: from
    /// here on the batch is finished, never undone.
    pub(super) placed: bool,
    pub(super) files: Vec<JournalFile>,
    /// Directories the batch created in the journal's folder, that folder
    /// included, outermost first.
    pub(super) dirs: Vec<PathBuf>,
}

/// A file of the batch. Only its path is kept: where its temp file and its
/// backup are is worked out again, never read, so a journal can't point the
/// repair at anything else.
#[derive(Serialize, Deserialize)]
pub(super) struct JournalFile {
    pub(super) path: PathBuf,
    /// Whether something was at `path` before the batch.
    pub(super) existed: bool,
    /// `path` is relative to the batch's second folder, not the journal's.
    #[serde(default)]
    pub(super) beside: bool,
}

impl Journal {
    /// The batch `manager` is about to run, nothing placed yet.
    pub(super) fn of(dir: &Path, manager: &PathManager) -> io::Result<Self> {
        let files = manager
            .items
            .values()
            .map(|item| {
                let (path, beside) = relative(dir, manager.beside.as_deref(), &item.path)?;
                Ok(JournalFile {
                    path,
                    existed: exists(&item.path)?,
                    beside,
                })
            })
            .collect::<io::Result<_>>()?;
        // Folders above the journal's own are not noted: made so that it could
        // exist, a repair leaves them, empty, where they are.
        let dirs = manager
            .created_dirs
            .iter()
            .filter_map(|created| created.strip_prefix(dir).ok())
            .map(Path::to_path_buf)
            .collect();
        Ok(Self {
            placed: false,
            files,
            dirs,
        })
    }
}

fn invalid(kind: io::ErrorKind, path: &Path, reason: &str) -> io::Error {
    io::Error::new(kind, runique_i18n::tf(reason, &[path.display()]))
}

/// `path` as seen from `dir`, or from the second folder (then `true`). A batch
/// with a journal keeps to its folders: a path outside could not be found
/// again from another working directory.
fn relative(dir: &Path, beside: Option<&Path>, path: &Path) -> io::Result<(PathBuf, bool)> {
    let (inside, second) = match (
        path.strip_prefix(dir),
        beside.map(|folder| path.strip_prefix(folder)),
    ) {
        (Ok(inside), _) => (inside, false),
        (Err(_), Some(Ok(inside))) => (inside, true),
        _ => {
            return Err(invalid(
                io::ErrorKind::InvalidInput,
                path,
                runique_i18n::keys::atomic::error::OUTSIDE_JOURNAL,
            ));
        }
    };
    if plain(inside) {
        Ok((inside.to_path_buf(), second))
    } else {
        Err(invalid(
            io::ErrorKind::InvalidInput,
            path,
            runique_i18n::keys::atomic::error::LEAVES_JOURNAL,
        ))
    }
}

/// Only names, one after the other: nothing that climbs (`..`) or starts over
/// from a root, so `dir.join(path)` stays under `dir`.
fn plain(path: &Path) -> bool {
    path.components()
        .all(|part| matches!(part, Component::Normal(_)))
}

/// Whether anything is at `path`, a dangling symlink included.
fn exists(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

/// Writes the journal whole: a cut leaves the previous one or this one, never
/// half of it.
pub(super) fn write(dir: &Path, journal: &Journal) -> io::Result<()> {
    let content = serde_json::to_vec_pretty(journal).map_err(io::Error::other)?;
    let temp = dir.join(TEMP_NAME);
    disk::write_temp(&temp, &content, false)?;
    fs::rename(&temp, dir.join(NAME))?;
    disk::sync_dir(dir)
}

/// The journal of `dir`, if a batch left one. It is read from the disk like
/// any other file: what it names is checked before anything is done with it.
pub(super) fn read(dir: &Path) -> io::Result<Option<Journal>> {
    let path = dir.join(NAME);
    let content = match fs::read(&path) {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let journal: Journal = serde_json::from_slice(&content).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {error}", path.display()),
        )
    })?;
    let named_file = |file: &JournalFile| {
        plain(&file.path) && file.path.file_name().is_some_and(|name| !is_reserved(name))
    };
    if journal.files.iter().all(named_file) && journal.dirs.iter().all(|created| plain(created)) {
        Ok(Some(journal))
    } else {
        Err(invalid(
            io::ErrorKind::InvalidData,
            &path,
            runique_i18n::keys::atomic::error::JOURNAL_FORGED,
        ))
    }
}

/// Marks the point of no return: every file is in place.
pub(super) fn mark_placed(dir: &Path) -> io::Result<()> {
    let Some(mut journal) = read(dir)? else {
        return Err(invalid(
            io::ErrorKind::NotFound,
            dir,
            runique_i18n::keys::atomic::error::NO_JOURNAL,
        ));
    };
    journal.placed = true;
    write(dir, &journal)
}

pub(super) fn remove(dir: &Path) -> io::Result<()> {
    disk::remove(&dir.join(TEMP_NAME))?;
    disk::remove(&dir.join(NAME))
}

/// Whether a batch was cut short in `dir` and still waits to be repaired.
/// For a command that reads the folder without writing in it: it can refuse
/// to go on, and leave the repair to the command that wrote.
pub fn interrupted(dir: &Path) -> bool {
    dir.join(NAME).exists()
}

/// Repairs the batch a power cut interrupted in `dir`, if there is one: the
/// project is put back as it was, or, when every file was already in place,
/// the cleanup is finished. Returns whether there was anything to repair.
///
/// To call before anything reads the folder: what follows then works on a
/// project that is whole. A repair that fails keeps the journal, so the next
/// call starts it again.
///
/// The repair only knows the states an interrupted batch can leave. Anything
/// else means someone has been there since: it then touches nothing and
/// returns every file that does not match, one per line, for the command to
/// show. Choosing between a file edited by hand and its original is not the
/// repair's call.
pub fn recover_from(dir: &Path) -> io::Result<bool> {
    recover_from_both(dir, None)
}

/// [`recover_from`], for a command whose batches may name files of a second
/// folder (`AtomicBuilder::beside`): `beside` is that folder, as the command
/// has it now.
pub fn recover_from_both(dir: &Path, beside: Option<&Path>) -> io::Result<bool> {
    let Some(mut repair) = examine(dir, beside)? else {
        return Ok(false);
    };
    for backup in &repair.spare_backups {
        disk::remove(backup)?;
    }
    if repair.placed {
        repair.manager.finish()?;
    } else {
        repair.manager.undo()?;
    }
    Ok(true)
}

/// What a repair is about to do, once every file has been looked at and
/// nothing touched yet.
pub(super) struct Repair {
    /// The batch as the cut left it: each file at the step whose rollback, or
    /// cleanup, does what the disk needs.
    pub(super) manager: PathManager,
    /// Every file was in place: the batch is finished, not undone.
    pub(super) placed: bool,
    /// Backups that hold the very content still at `path`: dropped first.
    pub(super) spare_backups: Vec<PathBuf>,
}

/// Reads the journal of `dir` and looks at every file it names, without
/// touching any: the whole disk is known to be in a state a cut can leave
/// before the first thing is put back.
pub(super) fn examine(dir: &Path, beside: Option<&Path>) -> io::Result<Option<Repair>> {
    let Some(journal) = read(dir)? else {
        return Ok(None);
    };
    let mut items = Vec::with_capacity(journal.files.len());
    let mut spare_backups = Vec::new();
    let mut unexpected = Vec::new();
    for file in &journal.files {
        let root = if file.beside {
            // Only a command that has a second folder writes such a journal.
            beside.ok_or_else(|| {
                invalid(
                    io::ErrorKind::InvalidData,
                    &dir.join(NAME),
                    runique_i18n::keys::atomic::error::JOURNAL_FORGED,
                )
            })?
        } else {
            dir
        };
        let path = root.join(&file.path);
        stays_inside(root, &path)?;
        let (temp_path, backup_path) = (temp_of(&path), backup_of(&path));
        let step = if journal.placed {
            AtomicStep::Done
        } else {
            match found(&path, &temp_path, &backup_path, file.existed)? {
                Ok(Found::Placed) if !file.existed => AtomicStep::Placed,
                Ok(Found::Placed | Found::Nothing | Found::Original) => AtomicStep::Prepared,
                Ok(Found::OriginalAndBackup) => {
                    spare_backups.push(backup_path.clone());
                    AtomicStep::Prepared
                }
                Err(reason) => {
                    unexpected.push(runique_i18n::tf(reason, &[path.display()]));
                    AtomicStep::Waiting
                }
            }
        };
        items.push((
            path.clone(),
            ItemsPath {
                path,
                temp_path,
                backup_path,
                private_file: false,
                source: SourceItems::Bytes(Vec::new()),
                step,
            },
        ));
    }
    if !journal.placed {
        unexpected.extend(strangers(dir, &journal)?);
    }
    if !unexpected.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            unexpected.join("\n"),
        ));
    }
    Ok(Some(Repair {
        manager: PathManager {
            items: items.into_iter().collect(),
            created_dirs: journal
                .dirs
                .iter()
                .map(|created| dir.join(created))
                .collect(),
            journal: Some(dir.to_path_buf()),
            beside: beside.map(Path::to_path_buf),
        },
        placed: journal.placed,
        spare_backups,
    }))
}

/// What a cut left at `path`.
enum Found {
    /// Nothing: the original, if there was one, is in the backup.
    Nothing,
    /// The batch's own content, the same as its temp file.
    Placed,
    /// The original, never reached.
    Original,
    /// The original, and a backup that holds the very same content: the cut
    /// came right after the backup was made.
    OriginalAndBackup,
}

/// Tells what is at `path` by comparing it with the batch's two other files,
/// or, as a translation key, why it is none of the states a cut can leave.
fn found(
    path: &Path,
    temp_path: &Path,
    backup_path: &Path,
    existed: bool,
) -> io::Result<Result<Found, &'static str>> {
    let backup = exists(backup_path)?;
    if !existed && backup {
        return Ok(Err(runique_i18n::keys::atomic::error::STRAY_BACKUP));
    }
    if !exists(path)? {
        return Ok(if existed && !backup {
            Err(runique_i18n::keys::atomic::error::GONE)
        } else {
            Ok(Found::Nothing)
        });
    }
    if disk::same(path, temp_path)? {
        return Ok(Ok(Found::Placed));
    }
    if !existed {
        return Ok(Err(runique_i18n::keys::atomic::error::NOT_CREATED));
    }
    if !backup {
        return Ok(Ok(Found::Original));
    }
    Ok(if disk::same(path, backup_path)? {
        Ok(Found::OriginalAndBackup)
    } else {
        Err(runique_i18n::keys::atomic::error::CHANGED)
    })
}

/// What sits in a folder the batch created without being one of its files:
/// that folder could then not be taken away with the rest.
fn strangers(dir: &Path, journal: &Journal) -> io::Result<Vec<String>> {
    let mut known = vec![dir.join(NAME), dir.join(TEMP_NAME)];
    // The folders the batch created are all under `dir`: so are their files.
    for file in journal.files.iter().filter(|file| !file.beside) {
        let path = dir.join(&file.path);
        known.extend([temp_of(&path), backup_of(&path), path]);
    }
    known.extend(journal.dirs.iter().map(|created| dir.join(created)));
    let mut strangers = Vec::new();
    for created in &journal.dirs {
        let entries = match fs::read_dir(dir.join(created)) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        for entry in entries {
            let path = entry?.path();
            if !known.contains(&path) {
                strangers.push(runique_i18n::tf(
                    runique_i18n::keys::atomic::error::STRANGER,
                    &[path.display()],
                ));
            }
        }
    }
    Ok(strangers)
}

/// A folder of the project replaced by a symlink would lead the repair out of
/// the journal's folder: where the file really is must still be under it.
fn stays_inside(dir: &Path, path: &Path) -> io::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    // A folder that can't be resolved, missing or unreachable, leads nowhere:
    // whatever looks at the file next fails on it, before anything is touched.
    let Ok(real) = fs::canonicalize(parent) else {
        return Ok(());
    };
    if real.starts_with(fs::canonicalize(dir)?) {
        Ok(())
    } else {
        Err(invalid(
            io::ErrorKind::InvalidData,
            path,
            runique_i18n::keys::atomic::error::MOVED_OUT,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TempDir;

    // Absent is an answer; a path that can't be looked at is an error.
    #[test]
    fn exists_tells_what_is_absent_from_what_it_cannot_reach() {
        let dir = TempDir::new();
        fs::write(dir.join("file"), "content").unwrap();

        assert!(exists(&dir.join("file")).unwrap());
        assert!(!exists(&dir.join("missing")).unwrap());
        assert!(exists(&dir.join("file/child")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn a_dangling_symlink_is_something_at_the_path() {
        let dir = TempDir::new();
        std::os::unix::fs::symlink(dir.join("nowhere"), dir.join("link")).unwrap();

        assert!(exists(&dir.join("link")).unwrap());
    }

    // A directory where the journal is expected: not "no journal", which would
    // let the command go on over a batch that may be waiting.
    #[test]
    fn a_journal_that_cannot_be_read_is_not_an_absent_one() {
        let dir = TempDir::new();
        fs::create_dir(dir.join(NAME)).unwrap();

        assert!(read(dir.path()).is_err());
        assert!(recover_from(dir.path()).is_err());
        assert!(dir.join(NAME).is_dir());
    }

    // A folder the batch created can't be opened: what is in it is unknown,
    // so the repair stops rather than take the folder for an empty one.
    #[cfg(unix)]
    #[test]
    fn a_folder_the_repair_cannot_look_into_stops_it() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new();
        fs::create_dir(dir.join("made")).unwrap();
        let journal = Journal {
            placed: false,
            files: Vec::new(),
            dirs: vec![PathBuf::from("made")],
        };
        write(dir.path(), &journal).unwrap();
        fs::set_permissions(dir.join("made"), fs::Permissions::from_mode(0o000)).unwrap();

        let outcome = recover_from(dir.path());
        fs::set_permissions(dir.join("made"), fs::Permissions::from_mode(0o755)).unwrap();

        assert_eq!(outcome.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        assert!(dir.join("made").is_dir());
        assert!(dir.join(NAME).exists());
    }
}
