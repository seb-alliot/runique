//! Routing — the `urlpatterns!` and `view!` macros, `Routes`, the registry of URL names, `RouterExt`.
pub mod get_post;
pub mod register_url;
pub mod router_ext;
pub mod routes;
pub mod urlpatterns;

pub use register_url::{
    UrlError, add_urls, register_name_url, register_pending, reverse, reverse_with_parameters,
};
pub use router_ext::RouterExt;
pub use routes::Routes;
