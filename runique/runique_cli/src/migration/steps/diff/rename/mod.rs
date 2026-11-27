//! What changed its name, and nothing else of what it is.
mod column;
mod enum_type;
mod enum_value;
mod table;

pub(crate) use column::column;
pub(crate) use enum_type::enum_types;
pub(crate) use enum_value::enum_values;
pub use table::effective_renames;
