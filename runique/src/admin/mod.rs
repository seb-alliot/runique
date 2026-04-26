//! Admin module — administration interface: routes, configuration, reloading daemon, permissions, forms.
pub mod admin_main;
pub mod config;
pub mod daemon;
pub mod helper;
pub mod middleware;
pub mod permissions;
pub mod registry;
pub mod resource;
pub mod router;
pub mod table_admin;
pub mod trad;

pub mod builtin;
pub mod forms;
pub mod history;

// Used by daemon-generated code in user projects (external crate) — must stay pub
pub use admin_main::{PrototypeAdminState, admin_get, admin_get_id, admin_post, admin_post_id};
pub use builtin::builtin_resources;
pub use config::AdminConfig;
pub use registry::AdminRegistry;
pub use resource::{
    AdminIdType, AdminResource, ColumnFilter, CrudOperation, DisplayConfig, ResourcePermissions,
};

pub use table_admin::migrations_table::*;

pub use permissions::{Groupe, pull_groupes_db};
pub(crate) use router::build_admin_router;
