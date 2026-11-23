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
