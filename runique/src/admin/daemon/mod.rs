pub mod generator;
pub mod parser;
pub mod watcher;

pub use generator::generate;
pub use parser::{ParsedAdmin, ResourceDef, parse_admin_file};
pub use watcher::watch;
