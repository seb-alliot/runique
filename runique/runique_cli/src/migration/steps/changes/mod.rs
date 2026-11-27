//! Working on the computed [`Changes`](crate::migration::types::Changes),
//! after `diff_schemas` and before any SQL is written: ordering them, and
//! inspecting them for destructive operations or names too long for the
//! engine. Laid out by action, as `diff` and `generate`.
mod create;
mod delete;
mod modify;
mod rename;

mod check_parallel;

pub mod destructive;
pub mod name_length_limit;
pub(crate) mod report;

pub use check_parallel::check_parallel;
pub use create::{cycle_keys, topological_sort_changes};
pub use delete::{drop_order, dropped_as_changes};
pub use destructive::*;
pub use modify::{
    decimal_may_shrink, length_may_shrink, primary_key_change, unmigrated_type_changes,
};
pub use name_length_limit::*;
pub use rename::{follow_renames, settle_renamed_keys};
