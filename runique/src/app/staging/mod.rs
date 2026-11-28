//! Staging steps — progressive configuration of the app before `build()` (core, middlewares, admin, static, CSP).
#[cfg(feature = "admin")]
pub mod admin;
mod applicator;
pub mod core;
pub mod middleware;
pub mod statics;

#[cfg(feature = "admin")]
pub use admin::AdminStaging;
pub use core::CoreStaging;
pub use middleware::MiddlewareStaging;
pub use statics::StaticStaging;
