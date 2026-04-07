//! Utilitaires de formulaires — parsing multipart/HTML et sanitisation HTML (ammonia).
pub mod parse_boolean;
pub mod parse_html;
pub mod sanitizer;

pub use parse_boolean::parse_bool;
pub use parse_html::*;
pub use sanitizer::*;
