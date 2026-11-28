//! `SessionStores`: the session stores the engine hands out, created once at
//! startup before the engine and shared with the middleware stack.
use crate::middleware::session::{CleaningMemoryStore, session_db::RuniqueSessionStore};
use std::sync::Arc;

/// The session stores built at startup. `None` when Runique doesn't own one:
/// no memory store when the developer plugged in their own
/// (`with_session_store`), no database store without the `orm` feature.
#[derive(Debug, Default)]
pub struct SessionStores {
    /// Anonymous sessions and CSRF, with the authenticated ones backed up in
    /// the database store.
    pub memory: Option<Arc<CleaningMemoryStore>>,
    /// Persistent authenticated sessions (table `eihwaz_sessions`).
    pub database: Option<Arc<RuniqueSessionStore>>,
}
