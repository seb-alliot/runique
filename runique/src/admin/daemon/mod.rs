//! Daemon de rechargement admin — génère `admin.rs` à chaud depuis la macro `admin!`.
pub(crate) mod generator;
pub(crate) mod parser;
pub(crate) mod watcher;

pub(crate) use generator::generate;
pub(crate) use parser::parse_admin_file;
pub(crate) use watcher::watch;
