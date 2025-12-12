//! Middleware components
pub mod error_handler;
pub mod tera_ext;

pub use error_handler::error_handler_middleware;
pub use tera_ext::TeraSafe;
