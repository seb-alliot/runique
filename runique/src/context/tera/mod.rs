//! Filtres et fonctions Tera — `form_filter`, `| static`, `{% link %}`, `| markdown`, CSRF token.
pub mod form;
pub mod static_tera;
pub mod url;

pub use form::*;
pub use static_tera::*;
pub use url::*;
