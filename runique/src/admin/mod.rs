//! Admin module — administration interface: routes, configuration, permissions.
pub mod config;
pub mod history;
pub mod middleware;
pub mod refonte_admin;
pub mod router;
pub mod trad;

pub use config::AdminConfig;
pub use refonte_admin::gate::operation::CrudOperation;
pub use router::admin_router::AdminState;
pub(crate) use router::build_admin_router;
pub use trad::{inject_admin_prefix, insert_admin_messages};
