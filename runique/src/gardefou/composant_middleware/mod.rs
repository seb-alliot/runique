// Déclaration des modules (fichiers .rs)
pub mod allowed_hosts;
pub mod csp_middleware;
pub mod csrf_middleware;
pub mod error_handler;
pub mod flash_message;
pub mod login_requiert;
pub mod middleware_sanitiser;

// Ré-exports pratiques
pub use allowed_hosts::AllowedHostsValidator;
pub use csp_middleware::*;
pub use csrf_middleware::*;
pub use error_handler::*;
pub use flash_message::*;
pub use login_requiert::*;
pub use middleware_sanitiser::*;
