//! A batch of files written all or nothing: each file is prepared beside its
//! target, placed, and the whole batch is undone on the first failure. With a
//! journal, a batch the power cut short is repaired at the next start.
//!
//! Written for the Runique framework (`runique new`, `makemigrations`,
//! `migration up`, uploaded files).
mod action;
pub mod builder;
mod disk;
mod journal;
mod manager;
pub mod private_file;
mod step;
mod struct_atomic;

pub use disk::remove;
pub use journal::{interrupted, recover_from, recover_from_both};
pub use struct_atomic::{AtomicBuilder, SourceItems};

#[cfg(any(test, feature = "test-support"))]
pub mod support;
#[cfg(test)]
pub(crate) mod tests;
