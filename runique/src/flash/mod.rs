//! Flash messages — session storage, levels (success/error/info/warning), Axum extractor.
pub mod flash_manager;
pub mod flash_struct;

pub use flash_manager::Message;
pub use flash_struct::*;
