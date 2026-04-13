//! Admin reload daemon — hot-generates `admin.rs` from the `admin!` macro.
pub(crate) mod generator;
pub(crate) mod parser;
pub(crate) mod watcher;

pub(crate) use generator::generate;
pub(crate) use parser::parse_admin_file;
pub(crate) use watcher::watch;
