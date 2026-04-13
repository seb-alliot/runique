//! Staging steps — progressive configuration of the app before `build()` (core, middlewares, admin, static, CSP).
pub mod admin_staging;
pub mod core_staging;
pub mod csp_config;
pub mod host_config;
pub mod middleware_staging;
pub mod static_staging;

pub use admin_staging::AdminStaging;
pub use core_staging::CoreStaging;
pub use csp_config::CspConfig;
pub use host_config::HostConfig;
pub use middleware_staging::MiddlewareStaging;
pub use static_staging::StaticStaging;
