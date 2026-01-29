// flash messages.rs
pub const FLASH_KEY: &str = "flash_messages";

// Csrf => pensé a changer le hardcode dans csrf_gate par la constante
pub const CSRF_TOKEN_KEY: &str = "csrf_token";

// csp.rs
pub const NONCE_KEY: &str = "csp_nonce";

/// Clé de session pour stocker l'ID utilisateur
pub const SESSION_USER_ID_KEY: &str = "user_id";
pub const SESSION_USER_USERNAME_KEY: &str = "username";
