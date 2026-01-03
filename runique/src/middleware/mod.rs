//! Middleware components
pub mod error_handler;
pub use error_handler::error_handler_middleware;

pub mod flash_message;
pub use flash_message::flash_middleware;

pub mod csrf;
pub use csrf::csrf_middleware;

pub mod middleware_sanetiser;
pub use middleware_sanetiser::sanitize_middleware;

pub mod csp;

pub mod login_requiert;

pub mod allowed_hosts;
pub use allowed_hosts::{AllowedHostsValidator, allowed_hosts_middleware};
