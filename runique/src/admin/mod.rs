//! Admin module — administration interface: routes, configuration, permissions.
pub mod builder;
pub mod config;
pub mod constants;
pub mod middleware;
pub mod router;
pub mod template;
pub mod translation;

pub use crate::auth::{gate::operation::CrudOperation, meta::ModelMeta};
pub use builder::{model_admin::ModelAdmin, site::AdminSite};
pub use config::AdminConfig;
pub(crate) use router::build_admin_router;
pub use router::{AdminView, admin_page_context, routes::AdminState};
pub use translation::{inject_admin_prefix, insert_admin_messages};
