//! Macros et helpers routeur — `get_post!`, `urlpatterns!`, registre des noms d'URL, `RouterExt`.
pub mod get_post;
pub mod register_url;
pub mod router;
pub mod router_ext;

pub use router_ext::RouterExt;
