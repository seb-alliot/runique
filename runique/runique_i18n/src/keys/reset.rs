//! Keys of `reset`.
pub mod error {
    pub const EMAIL_MISMATCH: &str = "reset.error.email_mismatch";
    pub const INVALID_OR_EXPIRED: &str = "reset.error.invalid_or_expired";
    pub const PASSWORD_MIN_LENGTH: &str = "reset.error.password_min_length";
    pub const PASSWORD_MISMATCH: &str = "reset.error.password_mismatch";
    pub const PASSWORD_WEAK: &str = "reset.error.password_weak";
    pub const UNAVAILABLE: &str = "reset.error.unavailable";
}
pub mod info {
    pub const CHECK_INBOX: &str = "reset.info.check_inbox";
}
pub mod label {
    pub const BLOCKED_BODY: &str = "reset.label.blocked_body";
    pub const BLOCKED_SUBJECT: &str = "reset.label.blocked_subject";
    pub const CONFIRM_LABEL: &str = "reset.label.confirm_label";
    pub const EMAIL_BODY: &str = "reset.label.email_body";
    pub const EMAIL_LABEL: &str = "reset.label.email_label";
    pub const EMAIL_SUBJECT: &str = "reset.label.email_subject";
    pub const FORGOT_TITLE: &str = "reset.label.forgot_title";
    pub const NEW_PASSWORD_LABEL: &str = "reset.label.new_password_label";
    pub const RESET_TITLE: &str = "reset.label.reset_title";
    pub const SUCCESS_TITLE: &str = "reset.label.success_title";
}
pub mod log {
    pub const PASSWORD_RESET_ASKED_BUT_NO_MAILER: &str =
        "reset.log.password_reset_asked_but_no_mailer";
    pub const PASSWORD_RESET_DB_ERROR: &str = "reset.log.password_reset_db_error";
    pub const PASSWORD_RESET_OK: &str = "reset.log.password_reset_ok";
    pub const PASSWORD_RESET_ROLLBACK_FAILED: &str = "reset.log.password_reset_rollback_failed";
    pub const RESET_EMAIL_SENT: &str = "reset.log.reset_email_sent";
    pub const RESET_LINK_NOT_SENT: &str = "reset.log.reset_link_not_sent";
    pub const RESET_TOKEN_CLEANUP_FAILED: &str = "reset.log.reset_token_cleanup_failed";
    pub const RESET_TOKEN_CONSUMPTION_COMMIT_FAILED: &str =
        "reset.log.reset_token_consumption_commit_failed";
    pub const RESET_TOKEN_CONSUMPTION_COULD_NOT_BEGIN: &str =
        "reset.log.reset_token_consumption_could_not_begin";
    pub const RESET_TOKEN_CONSUMPTION_FAILED: &str = "reset.log.reset_token_consumption_failed";
    pub const RESET_TOKEN_CONSUMPTION_ROLLBACK_FAILED: &str =
        "reset.log.reset_token_consumption_rollback_failed";
    pub const RESET_TOKEN_INVALID_EXPIRED: &str = "reset.log.reset_token_invalid_expired";
}
