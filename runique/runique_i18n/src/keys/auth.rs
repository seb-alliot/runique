//! Keys of `auth`.
pub mod error {
    pub const LOGIN_CANNOT_SIGN_IN: &str = "auth.error.login_cannot_sign_in";
    pub const LOGIN_SESSION_FAILED: &str = "auth.error.login_session_failed";
}
pub mod log {
    pub const ACCOUNT_NOT_READ: &str = "auth.log.account_not_read";
    pub const ADMIN_ACCOUNT_NOT_READ: &str = "auth.log.admin_account_not_read";
    pub const HASH_FAILED: &str = "auth.log.hash_failed";
}
