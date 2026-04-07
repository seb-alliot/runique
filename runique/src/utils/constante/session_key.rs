//! Clés de session et de formulaire — CSRF, flash, nonce CSP, user_id, rôles.
// Session protection key — timestamp Unix (i64) indiquant jusqu'à quand protéger la session.
// Posé manuellement par le dev pour des sessions anonymes à valeur (panier, formulaire multi-étapes).
// Le cleaner ne supprime pas les sessions où ce timestamp est dans le futur.
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
