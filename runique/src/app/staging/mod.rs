//! Staging steps — progressive configuration of the app before `build()` (core, middlewares, admin, static, CSP).
#[cfg(feature = "admin")]
pub mod admin;
mod applicator;
pub mod core_staging;
pub mod middleware;
pub mod static_staging;

#[cfg(feature = "admin")]
pub use admin::AdminStaging;
pub use core_staging::CoreStaging;
pub use middleware::MiddlewareStaging;
pub use static_staging::StaticStaging;
