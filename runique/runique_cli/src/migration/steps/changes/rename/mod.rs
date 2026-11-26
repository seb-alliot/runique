//! Renaming a table (`renamed_from`): what follows it.
mod follow_renames;
mod settle_renamed_keys;

pub use follow_renames::follow_renames;
pub use settle_renamed_keys::settle_renamed_keys;
