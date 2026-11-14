use crate::atomic::struct_atomic::{
    ActionAtomic, AtomicBuilder, AtomicStep, ItemsPath, PathManager, SourceItems,
};
use indexmap::IndexMap;
use std::{
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
        }
    }

    /// Adds a file to the batch. Adding the same `path` twice is an error.
    pub(crate) fn add(
        mut self,
        path: PathBuf,
        source: SourceItems,
        private_file: bool,
    ) -> io::Result<Self> {
        if self.items.contains_key(&path) {
            return Err(io::ErrorKind::AlreadyExists.into());
        }
        let item = ItemsPath {
            temp_path: with_suffix(&path, TEMP_SUFFIX),
            backup_path: with_suffix(&path, BACKUP_SUFFIX),
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
        };
        manager.run(self.strategy)
    }
}

/// `dir/name.ext` becomes `dir/name.ext.runique-tmp`: same directory, hence same filesystem.
fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".runique-{suffix}"));
    PathBuf::from(name)
}
