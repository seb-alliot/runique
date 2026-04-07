//! Constantes du framework — clés de session, CSRF, flash, templates, regex, erreurs.
pub mod admin_key;
pub mod error_key;
pub mod parse;
pub mod regex_template;
pub mod session_key;
pub mod template;

pub use admin_key::*;
pub use error_key::*;
pub use parse::*;
pub use regex_template::*;
pub use session_key::*;
pub use template::*;
