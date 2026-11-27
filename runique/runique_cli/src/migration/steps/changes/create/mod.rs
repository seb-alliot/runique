//! Creating: the order tables are created in, and the creation that may
//! lose data.
mod cascade_key;
mod cycle_keys;
mod table_order;

// What the destructive cases of this folder need: they take it by `use super::*`.
use super::destructive::{Destructive, DestructiveItem, found};
use crate::migration::types::Changes;

pub(crate) use cascade_key::cascade_key;
pub use cycle_keys::cycle_keys;
pub use table_order::topological_sort_changes;
