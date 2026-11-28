//! Keys of `mailer`.
pub mod error {
    pub const BUILD_FAILED: &str = "mailer.error.build_failed";
    pub const INVALID_RECIPIENT: &str = "mailer.error.invalid_recipient";
    pub const INVALID_REPLY_TO: &str = "mailer.error.invalid_reply_to";
    pub const INVALID_SENDER: &str = "mailer.error.invalid_sender";
    pub const NO_CONTENT: &str = "mailer.error.no_content";
    pub const NOT_CONFIGURED: &str = "mailer.error.not_configured";
    pub const SEND_FAILED: &str = "mailer.error.send_failed";
    pub const SMTP_CONNECTION: &str = "mailer.error.smtp_connection";
    pub const SUBJECT_LINE_BREAK: &str = "mailer.error.subject_line_break";
    pub const TEMPLATE: &str = "mailer.error.template";
}
pub mod label {
    pub const NO_CONTENT: &str = "mailer.label.no_content";
}
pub mod log {
    pub const INIT_TWICE: &str = "mailer.log.init_twice";
    pub const SEND: &str = "mailer.log.send";
    pub const SEND_ERROR: &str = "mailer.log.send_error";
    pub const SEND_MAILER_NOT_CONFIGURED: &str = "mailer.log.send_mailer_not_configured";
}
