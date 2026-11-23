//! Keys of `migrate`.
pub mod error {
    pub const CARGO_MISSING: &str = "migrate.error.cargo_missing";
    pub const CRATE_MISSING: &str = "migrate.error.crate_missing";
    pub const ENGINE_DEFAULTED: &str = "migrate.error.engine_defaulted";
    pub const ENGINE_UNSUPPORTED: &str = "migrate.error.engine_unsupported";
    pub const FAILED: &str = "migrate.error.failed";
    pub const INTERRUPTED: &str = "migrate.error.interrupted";
    pub const RENAMED_FROM_KEPT: &str = "migrate.error.renamed_from_kept";
    pub const SQLITE_KEY_CHANGE: &str = "migrate.error.sqlite_key_change";
}
pub mod info {
    pub const COMPLETE_DOWN: &str = "migrate.info.complete_down";
    pub const COMPLETE_UP: &str = "migrate.info.complete_up";
    pub const CRATE_CREATED: &str = "migrate.info.crate_created";
    pub const RUNNING: &str = "migrate.info.running";
}
