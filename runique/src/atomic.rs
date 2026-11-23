//! Batches of files written all or nothing: the `runique_atomic` crate.
pub(crate) use runique_atomic::{AtomicBuilder, SourceItems, remove};

#[cfg(test)]
pub(crate) mod tests {
    pub(crate) use runique_atomic::support::TempDir;
}
