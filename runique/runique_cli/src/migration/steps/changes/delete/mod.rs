//! Deleting: the order tables are dropped in, and what goes with a table,
//! a column or a key.
mod column;
mod key;
mod table;
mod table_order;

// What the destructive cases of this folder need: they take it by `use super::*`.
use super::destructive::{Destructive, DestructiveItem, found};
use crate::migration::types::Changes;

pub(crate) use column::column;
pub(crate) use key::key;
pub(crate) use table::table;
pub use table_order::drop_order;
