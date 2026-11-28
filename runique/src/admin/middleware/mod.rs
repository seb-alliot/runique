//! Admin middlewares — authentication required (`admin_access`) and permission verification.
pub(crate) mod access;
pub(crate) use access::{admin_access, declared_route_access};
