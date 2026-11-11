//! Working on the computed [`Changes`](crate::migration::types::Changes),
//! after `diff_schemas` and before any SQL is written: ordering them, and
//! inspecting them for destructive operations or names too long for the engine.
pub mod destructive_changes;
pub mod name_length_limit;
pub mod ordering;

pub use destructive_changes::*;
pub use name_length_limit::*;
pub use ordering::*;
