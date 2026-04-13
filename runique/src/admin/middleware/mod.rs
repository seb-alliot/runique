//! Admin middlewares — authentication required (`admin_required`) and permission verification.
pub(crate) mod admin_middleware;
pub(crate) use admin_middleware::admin_required;
