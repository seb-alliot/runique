//! Staging steps — progressive configuration of the app before `build()` (core, middlewares, admin, static, CSP).
pub mod admin_staging;
pub mod core_staging;
pub mod cors_config;
pub mod csp_config;
pub mod host_config;
pub mod middleware_staging;
pub mod permissions_policy_config;
pub mod static_staging;
pub mod trusted_proxies_config;

pub use admin_staging::AdminStaging;
pub use core_staging::CoreStaging;
pub use cors_config::CorsConfig;
pub use csp_config::CspConfig;
pub use host_config::HostConfig;
pub use middleware_staging::MiddlewareStaging;
pub use permissions_policy_config::PermissionsPolicyConfig;
pub use static_staging::StaticStaging;
pub use trusted_proxies_config::TrustedProxiesConfig;
