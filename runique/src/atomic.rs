//! Batches of files written all or nothing: the `runique_atomic` crate.
pub(crate) use runique_atomic::{AtomicBuilder, SourceItems, remove};

#[cfg(all(test, feature = "sqlite"))]
pub(crate) mod tests {
    pub(crate) use runique_atomic::support::TempDir;
}
