//! What a migration removes. Enum types are objects of their own under
//! PostgreSQL only.
mod enum_type;
mod key;

pub use enum_type::drop_enum_type;
pub use key::drop_foreign_key;
