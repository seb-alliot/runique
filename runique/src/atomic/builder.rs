use crate::atomic::{
    journal,
    struct_atomic::{ActionAtomic, AtomicBuilder, AtomicStep, ItemsPath, PathManager, SourceItems},
};
use indexmap::IndexMap;
use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

const TEMP_SUFFIX: &str = "tmp";
const BACKUP_SUFFIX: &str = "bak";

impl AtomicBuilder {
    /// A batch that only creates files: an existing `path` is an error.
    pub(crate) fn create() -> Self {
        Self::with(ActionAtomic::Create)
    }
    /// A batch that may replace existing files, keeping each original as a backup.
    pub(crate) fn modify() -> Self {
        Self::with(ActionAtomic::Modify)
    }

    fn with(strategy: ActionAtomic) -> Self {
        Self {
            strategy,
            items: IndexMap::new(),
            journal: None,
            beside: None,
        }
    }

    /// Keeps a journal in `dir` while the batch runs, so that a power cut can
    /// be repaired by `recover(dir)` at the next start. Every file of the batch
    /// must be under `dir`. For a command run by one person at a time: two
    /// batches at once in the same folder would share the journal.
    pub(crate) fn journal(mut self, dir: impl Into<PathBuf>) -> Self {
        self.journal = Some(dir.into());
        self
    }

    /// Lets the journal name files of a second folder, outside its own. The
    /// journal doesn't keep where that folder is: the command that repairs
    /// gives it again (`recover_beside`), so a journal can't send the repair
    /// anywhere the command didn't.
    pub(crate) fn beside(mut self, dir: impl Into<PathBuf>) -> Self {
        self.beside = Some(dir.into());
        self
    }

    /// Adds a file to the batch. Adding the same `path` twice is an error.
    pub(crate) fn add(
        mut self,
        path: PathBuf,
        source: SourceItems,
        private_file: bool,
    ) -> io::Result<Self> {
        // A batch than only creates files has nothing to take away.
        if matches!(source, SourceItems::Remove) && self.strategy == ActionAtomic::Create {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                crate::utils::trad::t("atomic.wrong_step").into_owned(),
            ));
        }
        // The names the batch works under are its own: a file of the batch
        // carrying one would be taken for the temp file or the backup of
        // another, and removed with it.
        match path.file_name() {
            Some(name) if !is_reserved(name) => {}
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    crate::utils::trad::tf("atomic.reserved_name", &[path.display()]),
                ));
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    crate::utils::trad::tf("atomic.not_a_file", &[path.display()]),
                ));
            }
        }
        if self.items.contains_key(&path) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                crate::utils::trad::tf("atomic.duplicate", &[path.display()]),
            ));
        }
        let item = ItemsPath {
            temp_path: temp_of(&path),
            backup_path: backup_of(&path),
            path: path.clone(),
            private_file,
            source,
            step: AtomicStep::Waiting,
        };
        self.items.insert(path, item);
        Ok(self)
    }

    /// Hands the batch over and runs it.
    pub(crate) fn start(self) -> io::Result<()> {
        let manager = PathManager {
            items: self.items,
            created_dirs: Vec::new(),
            journal: self.journal,
            beside: self.beside,
        };
        manager.run(self.strategy)
    }
}

/// Where the new content of `path` waits before it is placed.
pub(super) fn temp_of(path: &Path) -> PathBuf {
    with_suffix(path, TEMP_SUFFIX)
}

/// Where the original of `path` is kept while the batch runs.
pub(super) fn backup_of(path: &Path) -> PathBuf {
    with_suffix(path, BACKUP_SUFFIX)
}

/// Whether `name` is one the batch gives its own files.
pub(super) fn is_reserved(name: &OsStr) -> bool {
    let name = name.to_string_lossy();
    [TEMP_SUFFIX, BACKUP_SUFFIX]
        .iter()
        .any(|suffix| name.ends_with(&format!(".runique-{suffix}")))
        || name == journal::NAME
}

/// `dir/name.ext` becomes `dir/name.ext.runique-tmp`: same directory, hence same filesystem.
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".runique-{suffix}"));
    PathBuf::from(name)
}
