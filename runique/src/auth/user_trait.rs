//! `RuniqueUser` trait: minimal contract for any Runique user model.

/// Contract to implement on any user model for framework integration.
pub trait RuniqueUser: Send + Sync {
    /// Primary key of the user record.
    fn user_id(&self) -> crate::utils::pk::Pk;
    /// Login name (unique).
    fn username(&self) -> &str;
    /// Email address (unique).
    fn email(&self) -> &str;
    /// Whether the account is enabled. Inactive users cannot log in.
    fn is_active(&self) -> bool;
    /// Full admin access — bypasses all permission checks.
    fn is_superuser(&self) -> bool;

    /// Whether the account may sign in. Default: `is_active()`. The built-in
    /// model also requires the account to have been activated.
    fn can_sign_in(&self) -> bool {
        self.is_active()
    }

    /// Increases to close every session of the account, whatever store holds
    /// them. Default: `0`, which never closes anything; the built-in model
    /// reads it from `eihwaz_users`.
    fn session_version(&self) -> i32 {
        0
    }
}
