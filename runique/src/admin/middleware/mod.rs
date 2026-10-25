//! Admin middlewares — authentication required (`admin_access`) and permission verification.
pub(crate) mod admin_middleware;
pub(crate) use admin_middleware::{admin_access, declared_route_access};
