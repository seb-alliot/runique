//! Keys of `server`.
pub mod error {
    pub const ACME_AUTHORIZATION_STATUS: &str = "server.error.acme_authorization_status";
    pub const ACME_NOT_COMPILED: &str = "server.error.acme_not_compiled";
    pub const ACME_ORDER_NOT_READY: &str = "server.error.acme_order_not_ready";
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
    pub const ACME_RENEWAL_FAILED_KEEPING_CURRENT_CERTIFICATE: &str =
        "server.log.acme_renewal_failed_keeping_current_certificate";
    pub const ACME_RENEWAL_FAILED_USING_EXISTING_CERTIFICATE: &str =
        "server.log.acme_renewal_failed_using_existing_certificate";
    pub const CERTIFICATE_DOES_NOT_COVER_DOMAIN: &str =
        "server.log.certificate_does_not_cover_domain";
    pub const CERTIFICATE_MISSING_UNREADABLE: &str = "server.log.certificate_missing_unreadable";
    pub const CERTIFICATE_NOT_VALID_PEM: &str = "server.log.certificate_not_valid_pem";
    pub const CERTIFICATE_NOT_VALID_X_509_CERTIFICATE: &str =
        "server.log.certificate_not_valid_x_509_certificate";
    pub const CERTIFICATE_OBTAINED_STORED: &str = "server.log.certificate_obtained_stored";
    pub const CERTIFICATE_OBTAINED_SUCCESSFULLY: &str =
        "server.log.certificate_obtained_successfully";
    pub const CTRL_C_NOT_INSTALLED: &str = "server.log.ctrl_c_not_installed";
    pub const HTTPS_PORT80_STOPPED: &str = "server.log.https_port80_stopped";
    pub const HTTPS_REDIRECTED: &str = "server.log.https_redirected";
    pub const LOADING_EXISTING_TLS_CERTIFICATE: &str =
        "server.log.loading_existing_tls_certificate";
    pub const RENEWED_CERTIFICATE_NOT_LOADED_KEEPING_CURRENT: &str =
        "server.log.renewed_certificate_not_loaded_keeping_current";
    pub const SIGTERM_NOT_INSTALLED: &str = "server.log.sigterm_not_installed";
    pub const STARTING_ACME_CERTIFICATE_PROVISIONING: &str =
        "server.log.starting_acme_certificate_provisioning";
    pub const TLS_CERTIFICATE_RENEWED_RELOADED: &str =
        "server.log.tls_certificate_renewed_reloaded";
}
