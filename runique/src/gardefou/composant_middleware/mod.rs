// Déclaration des modules (fichiers .rs)
pub mod allowed_hosts;
pub mod csp;
pub mod csrf;
pub mod error_handler;
pub mod flash_message;
pub mod login_requiert;
pub mod middleware_sanitiser;

// Ré-exports pratiques
pub use allowed_hosts::AllowedHostsValidator;
pub use csp::*;
pub use csrf::*;
pub use error_handler::*;
pub use flash_message::*;
pub use login_requiert::*;
pub use middleware_sanitiser::*;
