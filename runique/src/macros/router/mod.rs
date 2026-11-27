//! Router macros and helpers — `get_post!`, `urlpatterns!`, URL name registry, `RouterExt`.
pub mod get_post;
pub mod register_url;
pub mod router_ext;
pub mod routes;
pub mod urlpatterns;

pub use router_ext::RouterExt;
pub use routes::Routes;
