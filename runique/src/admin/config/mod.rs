//! Admin interface configuration — `AdminConfig` (title, auth, pagination, roles...).
pub mod config_admin;
pub mod template;
pub use config_admin::AdminConfig;
pub(crate) use template::AdminTemplate;
