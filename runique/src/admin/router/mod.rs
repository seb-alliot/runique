//! Admin router — `build_admin_router`, `AdminState`, template session key.
pub mod admin_router;
pub use admin_router::{ADMIN_TEMPLATE_SESSION_KEY, AdminState, build_admin_router};
