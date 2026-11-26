//! What a migration changes in place: one function per case, one file per function.
mod column;
mod column_type;
mod enum_column;

pub(crate) use column::column;
pub(crate) use column_type::column_type;
pub(crate) use enum_column::enum_column;
