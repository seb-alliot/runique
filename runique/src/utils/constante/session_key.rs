//! Session and form keys — CSRF, flash, CSP nonce, user_id, roles.
// Session protection key — Unix timestamp (i64) indicating until when the session should be protected.
// Set manually by the dev for anonymous sessions with value (cart, multi-step form).
// The cleaner does not delete sessions where this timestamp is in the future.
/// Keys used to store data in the `tower_sessions` session store, and a couple
/// of related form field names.
pub mod session {
    /// Session key holding pending flash messages (see `flash::flash_manager`).
    pub const FLASH_KEY: &str = "flash_messages";
    /// Session key storing the CSRF token for the session; also the name of the
    /// hidden form field / header carrying it back on submit.
    pub const CSRF_TOKEN_KEY: &str = "csrf_token";
    /// Session key storing the per-request CSP nonce.
    pub const NONCE_KEY: &str = "csp_nonce";
    /// Session key storing the authenticated user's id.
    pub const SESSION_USER_ID_KEY: &str = "user_id";
    /// Session key storing the Unix timestamp until which an anonymous session
    /// is protected from cleanup. Set manually by the app for anonymous sessions
    /// carrying value (cart, multi-step form) — the session cleaner skips any
    /// session where this timestamp is still in the future.
    pub const SESSION_ACTIVE_KEY: &str = "session_active";
    /// Session key storing the authenticated user's username.
    pub const SESSION_USER_USERNAME_KEY: &str = "username";
    /// Session key storing whether the authenticated user is staff.
    pub const SESSION_USER_IS_STAFF_KEY: &str = "is_staff";
    /// Session key storing whether the authenticated user is a superuser.
    pub const SESSION_USER_IS_SUPERUSER_KEY: &str = "is_superuser";
    /// Session key storing the authenticated user's roles.
    pub const SESSION_USER_ROLES_KEY: &str = "roles";
    /// Session key storing the authenticated user's droits (permissions).
    pub const SESSION_USER_DROITS_KEY: &str = "droits";
    /// Form field name for the `is_active` flag on the built-in user admin form.
    pub const IS_ACTIVE: &str = "is_active";
    /// Name of the honeypot form field checked by the anti-bot middleware —
    /// legitimate users leave it empty, so a non-empty value flags a bot.
    pub const HP_FIELD_KEY: &str = "_hp";
}
