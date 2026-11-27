//! Modifying: the primary key, the changes of a column that may lose data,
//! and the type changes that are not migrated yet.
mod column_decimal;
mod column_length;
mod column_required;
mod column_type;
mod decimal_may_shrink;
mod length_may_shrink;
mod primary_key;
mod unmigrated_type_changes;

// What the destructive cases of this folder need: they take it by `use super::*`.
use super::destructive::{Destructive, DestructiveItem, found};
use crate::migration::types::Changes;

pub(crate) use column_decimal::column_decimal;
pub(crate) use column_length::column_length;
pub(crate) use column_required::column_required;
pub(crate) use column_type::column_type;
pub use decimal_may_shrink::decimal_may_shrink;
pub use length_may_shrink::length_may_shrink;
pub use primary_key::primary_key_change;
pub use unmigrated_type_changes::unmigrated_type_changes;
