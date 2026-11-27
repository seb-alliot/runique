//! Framework macros — DB (objects), Tera context, forms, router, templates.
pub mod context;
pub mod database;
pub mod forms;
pub mod helper {
    pub use super::context::helper::*;
}
pub mod router;
pub mod template;

pub use router::RouterExt;
pub use router::Routes;
pub use router::register_url::{
    UrlError, add_urls, register_name_url, register_pending, reverse, reverse_with_parameters,
};
