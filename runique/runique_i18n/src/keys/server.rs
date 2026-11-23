//! Keys of `server`.
pub mod error {
    pub const ACME_NOT_COMPILED: &str = "server.error.acme_not_compiled";
}
pub mod hint {
    pub const STOP: &str = "server.hint.stop";
}
pub mod info {
    pub const ACME_ADD_FEATURE: &str = "server.info.acme_add_feature";
    pub const CONNECTED_DB: &str = "server.info.connected_db";
    pub const CONNECTED_ENGINE: &str = "server.info.connected_engine";
    pub const LAUNCHED: &str = "server.info.launched";
    pub const OPERATIONAL: &str = "server.info.operational";
    pub const SHUTTING_DOWN: &str = "server.info.shutting_down";
    pub const TEST_UTILS_FEATURE_ENABLED: &str = "server.info.test_utils_feature_enabled";
}
pub mod log {
    pub const CTRL_C_NOT_INSTALLED: &str = "server.log.ctrl_c_not_installed";
    pub const HTTPS_PORT80_STOPPED: &str = "server.log.https_port80_stopped";
    pub const HTTPS_REDIRECTED: &str = "server.log.https_redirected";
    pub const SIGTERM_NOT_INSTALLED: &str = "server.log.sigterm_not_installed";
}
