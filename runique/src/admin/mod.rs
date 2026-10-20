//! Admin module — administration interface: routes, configuration, permissions.
pub mod builder;
pub mod config;
pub mod gate;
pub mod history;
pub mod meta;
pub mod middleware;
pub mod router;
pub mod trad;

pub use builder::{admin_builder::ModelAdmin, site::AdminSite};
pub use config::AdminConfig;
pub use gate::operation::CrudOperation;
pub use meta::ModelMeta;
pub use router::admin_router::AdminState;
pub(crate) use router::build_admin_router;
pub use trad::{inject_admin_prefix, insert_admin_messages};
