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

/// Carries the admin CRUD router and the path it is built for.
///
/// Returned by the daemon-generated `admins::routes(path)`.
/// Passed to `AdminStaging::routes()`, which records both.
///
/// `path` is the admin's own location (`/site-admin`), **not** a prefix: the
/// segment added in front of it comes from `AdminStaging::prefix()` and is
/// applied separately at mount time. Naming this field `prefix` made the two
/// look like the same setting when they appear side by side in a builder chain.
pub struct AdminRoutes {
    pub router: axum::Router,
    pub path: String,
}

impl AdminRoutes {
    pub fn new(path: impl Into<String>, router: axum::Router) -> Self {
        Self {
            router,
            path: path.into(),
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
