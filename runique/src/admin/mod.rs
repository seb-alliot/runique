pub mod config;
pub mod daemon;
pub mod middleware;
pub mod registry;
pub mod resource;
pub mod router;

pub use config::AdminConfig;
pub use daemon::{generate, parse_admin_file, watch};
pub use registry::AdminRegistry;
pub use resource::{
    AdminResource, ColumnFilter, CrudOperation, DisplayConfig, ResourcePermissions,
};
pub use router::build_admin_router;
pub use router::AdminState;
