pub mod generator;
pub mod parser;
pub mod watcher;

pub use generator::generate;
pub use parser::{parse_admin_file, ParsedAdmin, ResourceDef};
pub use watcher::watch;
