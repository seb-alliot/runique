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
pub use admin_main::{
    PrototypeAdminState, admin_get, admin_get_id, admin_nested_get, admin_nested_get_id,
    admin_nested_post, admin_nested_post_id, admin_post, admin_post_id,
};
pub use builtin::builtin_resources;
pub use config::AdminConfig;
pub use helper::{fetch_fk_label_map, fk_key, resolve_fk_labels, resolve_fk_labels_in_rows};

/// Carries the admin CRUD router and its URL prefix.
///
/// Returned by the daemon-generated `admins::routes(prefix)`.
/// Passed to `AdminStaging::routes()` which sets both the router and prefix automatically.
pub struct AdminRoutes {
    pub router: axum::Router,
    pub prefix: String,
}

impl AdminRoutes {
    pub fn new(prefix: impl Into<String>, router: axum::Router) -> Self {
        Self {
            router,
            prefix: prefix.into(),
        }
    }

    pub fn merge(mut self, other: axum::Router) -> Self {
        self.router = self.router.merge(other);
        self
    }
}
pub use registry::AdminRegistry;
pub use resource::{
    AdminIdType, AdminResource, ColumnFilter, CrudOperation, DisplayConfig, ParentScope,
    ResourcePermissions,
};

pub use table_admin::migrations_table::*;

pub use permissions::{Groupe, pull_groupes_db};
pub use router::admin_router::AdminState;
pub(crate) use router::build_admin_router;
pub use trad::{inject_admin_prefix, insert_admin_messages};
