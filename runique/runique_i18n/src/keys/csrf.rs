//! Keys of `csrf`.
pub mod error {
    pub const CROSS_SITE: &str = "csrf.error.cross_site";
    pub const INVALID: &str = "csrf.error.invalid";
    pub const INVALID_OR_MISSING: &str = "csrf.error.invalid_or_missing";
    pub const MISSING: &str = "csrf.error.missing";
    pub const REQUIRED: &str = "csrf.error.required";
    pub const SESSION_WRITE: &str = "csrf.error.session_write";
}
pub mod log {
    pub const KEEP_TOKEN: &str = "csrf.log.keep_token";
    pub const READ_ACCOUNT: &str = "csrf.log.read_account";
    pub const READ_TOKEN: &str = "csrf.log.read_token";
    pub const REASON_ADMIN_LOGIN_MISMATCH: &str = "csrf.log.reason_admin_login_mismatch";
    pub const REASON_ADMIN_LOGIN_MISSING: &str = "csrf.log.reason_admin_login_missing";
    pub const REASON_ADMIN_LOGOUT: &str = "csrf.log.reason_admin_logout";
    pub const REASON_CROSS_SITE: &str = "csrf.log.reason_cross_site";
    pub const REASON_FORM_MISMATCH: &str = "csrf.log.reason_form_mismatch";
    pub const REASON_FORM_MISSING: &str = "csrf.log.reason_form_missing";
    pub const REASON_HEADER_INVALID: &str = "csrf.log.reason_header_invalid";
    pub const REASON_NOT_A_FORM: &str = "csrf.log.reason_not_a_form";
    pub const REASON_UPLOAD: &str = "csrf.log.reason_upload";
    pub const REFUSED: &str = "csrf.log.refused";
    pub const SET_VALUE: &str = "csrf.log.set_value";
    pub const TOKEN_DROPPED: &str = "csrf.log.token_dropped";
    pub const TOKEN_NEW: &str = "csrf.log.token_new";
    pub const WRITE_TOKEN: &str = "csrf.log.write_token";
}
