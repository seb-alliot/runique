use indexmap::IndexMap;
use std::path::PathBuf;

/// What can be asked of a file in the batch.
/// Shared by `runique new` (create) and `makemigrations` (create or modify).
pub(crate) enum ActionAtomic {
    /// Create a file that must not exist yet.
    Create,
    /// Undo everything after an error and bring the file back to its initial state.
    Rollback,
    /// Create a file or replace an existing one, keeping the original as a backup.
    Modify,
    /// Final cleanup once the whole batch is `Done`: removes backups and temp files.
    Finish,
}

/// How far a file of the batch has progressed.
pub(crate) enum AtomicStep {
    /// Nothing has been done for this file.
    Waiting,
    /// The temp file is on disk (`temp_path`), `path` is untouched.
    Prepared,
    /// Swapped in: the previous file sits in `backup_path`, to be restored on rollback.
    Replaced,
    /// Swapped in: nothing was there before, to be removed on rollback.
    Placed,
    /// The file is at its final location. Backup and temp file stay until `Finish`.
    Done,
}

/// Where the content of a file comes from.
pub(crate) enum SourceItems {
    /// Content already in memory: `runique new`, `makemigrations`.
    Bytes(Vec<u8>),
    /// File already on the same filesystem (staged upload): linked, never loaded in memory.
    File(PathBuf),
}

/// One file of the batch: its three paths, its content, its step.
pub(crate) struct ItemsPath {
    pub(super) path: PathBuf,
    pub(super) backup_path: PathBuf,
    pub(super) temp_path: PathBuf,
    pub(super) private_file: bool,
    pub(super) source: SourceItems,
    pub(super) step: AtomicStep,
}

/// Holds the paths of the files and directories of a batch.
pub(crate) struct PathManager {
    /// Files of the batch in insertion order: the key is the path, the position is the index.
    pub(super) items: IndexMap<PathBuf, ItemsPath>,
    /// Directories created by the batch, to be removed on rollback.
    pub(super) created_dirs: Vec<PathBuf>,
}

/// Describes a batch: the action to apply and the files it applies to.
pub(crate) struct AtomicBuilder {
    pub(super) strategy: ActionAtomic,
    /// Adding the same path twice is caught at insertion.
    pub(super) items: IndexMap<PathBuf, ItemsPath>,
}
