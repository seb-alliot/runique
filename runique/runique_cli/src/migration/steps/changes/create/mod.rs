//! Creating: the order tables are created in, and the creation that may
//! lose data.
mod cascade_key;
mod table_order;

// What the destructive cases of this folder need: they take it by `use super::*`.
use super::destructive::{Destructive, DestructiveKind, found};
use crate::migration::types::Changes;

pub(crate) use cascade_key::cascade_key;
pub use table_order::topological_sort_changes;
