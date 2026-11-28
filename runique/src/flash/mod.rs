//! Flash messages — session storage, levels (success/error/info/warning), Axum extractor.
pub mod constants;
pub mod extractor;
pub mod message;

pub use extractor::Message;
pub use message::*;
