//! Session and form keys — CSRF, flash, CSP nonce, user_id, roles.
// Session protection key — Unix timestamp (i64) indicating until when the session should be protected.
// Set manually by the dev for anonymous sessions with value (cart, multi-step form).
// The cleaner does not delete sessions where this timestamp is in the future.
pub mod session {
    pub const FLASH_KEY: &str = "flash_messages";
    pub const CSRF_TOKEN_KEY: &str = "csrf_token";
    pub const NONCE_KEY: &str = "csp_nonce";
    pub const SESSION_USER_ID_KEY: &str = "user_id";
    pub const SESSION_ACTIVE_KEY: &str = "session_active";
    pub const SESSION_USER_USERNAME_KEY: &str = "username";
    pub const SESSION_USER_IS_STAFF_KEY: &str = "is_staff";
    pub const SESSION_USER_IS_SUPERUSER_KEY: &str = "is_superuser";
    pub const SESSION_USER_ROLES_KEY: &str = "roles";
    pub const SESSION_USER_DROITS_KEY: &str = "droits";
    pub const IS_ACTIVE: &str = "is_active";
}
