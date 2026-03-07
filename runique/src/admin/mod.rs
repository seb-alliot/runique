pub use cli_admin::create_superuser;
pub mod admin_main;
pub mod cli_admin;
pub mod config;
pub mod daemon;
pub mod dyn_form;
pub mod middleware;
pub mod registry;
pub mod resource;
pub mod resource_entry;
pub mod router;
pub mod template;

pub use admin_main::{admin_get, admin_get_id, admin_post, admin_post_id, PrototypeAdminState};
pub use config::AdminConfig;
pub use daemon::{generate, parse_admin_file, watch};
pub use dyn_form::DynForm;
pub use registry::AdminRegistry;
pub use resource::{
    AdminResource, ColumnFilter, CrudOperation, DisplayConfig, ResourcePermissions,
};
pub use resource_entry::{
    CountFn, CreateFn, DeleteFn, FormBuilder, GetFn, ListFn, ResourceEntry, UpdateFn,
};
pub use router::build_admin_router;
pub use router::AdminState;
pub use template::{AdminTemplate, PathAdminTemplate};
