//! What a migration adds: a key to an existing table, an enum, a value to
//! an enum.
mod enum_type;
mod enum_value;
mod key;

pub use enum_type::create_enum_type;
pub use enum_value::add_enum_value;
pub use key::add_foreign_key;
