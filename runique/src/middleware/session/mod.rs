//! Session management — stores (memory, DB), automatic cleanup, and the two
//! middlewares that run right after the session layer.
pub(crate) mod auth;
pub mod cleaning_store;
pub mod session_db;
pub(crate) mod ttl_upgrade;

pub use cleaning_store::CleaningMemoryStore;
pub use session_db::RuniqueSessionStore;

pub(crate) use auth::auth_middleware;
pub(crate) use ttl_upgrade::session_ttl_upgrade;
