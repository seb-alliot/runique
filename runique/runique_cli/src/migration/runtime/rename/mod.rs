//! What a migration renames.
mod enum_type;
mod enum_value;
mod key;

pub use enum_type::rename_enum_type;
pub use enum_value::rename_enum_value;
pub use key::rename_foreign_key;
