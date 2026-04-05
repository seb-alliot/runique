//! Daemon de rechargement admin — génère `admin.rs` à chaud depuis la macro `admin!`.
pub mod generator;
pub mod parser;
pub mod watcher;

pub use generator::generate;
pub use parser::{ConfigureDef, ParsedAdmin, ResourceDef, parse_admin_file};
pub use watcher::watch;
