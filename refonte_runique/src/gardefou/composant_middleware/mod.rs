// Déclaration des modules (fichiers .rs)
pub mod allowed_hosts;
pub mod csp;
pub mod csrf;
pub mod error_handler;
pub mod flash_message;
pub mod login_requiert;
pub mod middleware_sanitiser;


// Ré-exportations pour un accès simplifié depuis le reste du framework
// Cela permet de faire : use crate::gardefou::composant_middleware::csrf_middleware;

pub use allowed_hosts::AllowedHostsValidator;
pub use csp::{csrf_middleware, CspConfig};
pub use error_handler::error_handler_middleware;
pub use flash_message::flash_middleware;
pub use login_requiert::{load_user_middleware, is_authenticated};
pub use middleware_sanitiser::sanitize_middleware;