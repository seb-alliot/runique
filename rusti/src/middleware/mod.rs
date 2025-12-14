//! Middleware components
pub mod error_handler;
pub use error_handler::error_handler_middleware;

pub mod flash_message;
pub use flash_message::flash_middleware;
pub use flash_message::{flash_error, flash_info, flash_success};
