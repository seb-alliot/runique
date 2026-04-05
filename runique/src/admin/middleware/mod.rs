//! Middlewares admin — authentification requise (`admin_required`) et vérification de permission.
pub(crate) mod admin_middleware;
pub(crate) use admin_middleware::admin_required;
