//! Middlewares admin — authentification requise (`admin_required`) et vérification de permission.
pub mod admin_middleware;
pub use admin_middleware::{admin_required, check_permission};
