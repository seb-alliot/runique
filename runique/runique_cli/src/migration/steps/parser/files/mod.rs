//! What is read off the disk: the entities of the project, its Rust files,
//! its snapshots, the type of its primary keys.
pub(crate) mod enums_of;
pub(crate) mod project_pk;
pub(crate) mod rust_files;
pub mod scan;
pub(crate) mod snapshot;
