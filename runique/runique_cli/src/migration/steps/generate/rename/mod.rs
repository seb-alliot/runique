//! What a migration renames: one function per object, one file per function.
mod column;
mod enum_type;
mod enum_value;
mod index;
mod key;
mod table;

pub(crate) use column::column;
pub(crate) use enum_type::enum_type;
pub(crate) use enum_value::enum_value;
pub(crate) use index::index;
pub(crate) use key::key;
pub(crate) use table::table;
