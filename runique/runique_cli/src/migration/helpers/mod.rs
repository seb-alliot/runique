//! Small functions several steps of the migration engine share: each
//! answers one question about a name, a column, a schema or a generated
//! file, and belongs to no step in particular.
pub mod columns;
pub mod file;
pub mod markers;
pub mod naming;
pub mod schemas;

pub use columns::{decimal_may_shrink, length_may_shrink};
pub use markers::{has_marker, without_marker};
pub use naming::to_snake_case;
pub use schemas::db_columns;
