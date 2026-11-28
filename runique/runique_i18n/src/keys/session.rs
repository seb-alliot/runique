//! Keys of `session`.
pub mod error {
    pub const CREATE_EMPTY_COOKIE: &str = "session.error.create_empty_cookie";
    pub const ENCRYPT_FAILED: &str = "session.error.encrypt_failed";
    pub const UPSERT_EMPTY_COOKIE: &str = "session.error.upsert_empty_cookie";
}
pub mod log {
    pub const AUTH_SESSION_TTL_ALREADY_SET_DIFFERENT: &str =
        "session.log.auth_session_ttl_already_set_different";
    pub const AUTH_SESSION_TTL_SET_RACED_BUILD: &str =
        "session.log.auth_session_ttl_set_raced_build";
    pub const DECODE_USER_DB: &str = "session.log.decode_user_db";
    pub const DECODE_USER_MEMORY: &str = "session.log.decode_user_memory";
    pub const DELETE_SESSION_DB_FAILED: &str = "session.log.delete_session_db_failed";
    pub const EXCLUSIVE_LOGIN_SESSION_S_INVALIDATED_USER: &str =
        "session.log.exclusive_login_session_s_invalidated_user";
    pub const HIGH_WATERMARK_BYTES_FREED_EMERGENCY: &str =
        "session.log.high_watermark_bytes_freed_emergency";
    pub const LARGE_SESSION_RECORD_BYTES_AVOID_STORING: &str =
        "session.log.large_session_record_bytes_avoid_storing";
    pub const LOW_WATERMARK_BYTES_FREED_EXPIRED_ANONYMOUS: &str =
        "session.log.low_watermark_bytes_freed_expired_anonymous";
    pub const READ_ACCOUNT: &str = "session.log.read_account";
    pub const READ_DASHBOARD_TEMPLATE: &str = "session.log.read_dashboard_template";
    pub const READ_FLASH: &str = "session.log.read_flash";
    pub const READ_VERSION: &str = "session.log.read_version";
    pub const SESSION_CLEANUP_ERROR: &str = "session.log.session_cleanup_error";
    pub const SESSION_DB_BACKUP_UPDATE_FAILED: &str = "session.log.session_db_backup_update_failed";
    pub const SESSION_DB_BACKUP_WRITE_FAILED: &str = "session.log.session_db_backup_write_failed";
    pub const SESSION_DB_FALLBACK_LOAD_FAILED_COOKIE: &str =
        "session.log.session_db_fallback_load_failed_cookie";
    pub const SESSION_DB_VALIDITY_CHECK_FAILED_MEMORY: &str =
        "session.log.session_db_validity_check_failed_memory";
    pub const SESSION_STORE_SATURATED_BYTES_NEW_SESSION: &str =
        "session.log.session_store_saturated_bytes_new_session";
    pub const STORED_SESSION_DATA_COULD_NOT_DECRYPTED: &str =
        "session.log.stored_session_data_could_not_decrypted";
}
