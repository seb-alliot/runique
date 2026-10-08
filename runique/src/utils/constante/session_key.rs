//! Session keys (CSRF, flash, signed-in user id, protection) and a few
//! form field names.
/// Keys used to store data in the `tower_sessions` session store, and a couple
/// of related form field names.
pub mod session {
    /// Session key holding pending flash messages (see `flash::flash_manager`).
    pub const FLASH_KEY: &str = "flash_messages";
    /// Session key storing the CSRF token for the session; also the name of the
    /// hidden form field / header carrying it back on submit.
    pub const CSRF_TOKEN_KEY: &str = "csrf_token";
    /// Session key storing the authenticated user's id.
    pub const SESSION_USER_ID_KEY: &str = "user_id";
    /// Session key storing the Unix timestamp until which an anonymous session
    /// is protected from cleanup. Set manually by the app for anonymous sessions
    /// carrying value (cart, multi-step form) — the session cleaner skips any
    /// session where this timestamp is still in the future.
    pub const SESSION_ACTIVE_KEY: &str = "session_active";
    /// Name of the honeypot form field checked by the anti-bot middleware —
    /// legitimate users leave it empty, so a non-empty value flags a bot.
    pub const HP_FIELD_KEY: &str = "_hp";
}
