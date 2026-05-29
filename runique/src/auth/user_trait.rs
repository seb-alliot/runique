//! `RuniqueUser` trait: minimal contract for any Runique user model.

/// Contract to implement on any user model for framework integration.
pub trait RuniqueUser: Send + Sync {
    /// Primary key of the user record.
    fn user_id(&self) -> crate::utils::pk::Pk;
    /// Login name (unique).
    fn username(&self) -> &str;
    /// Email address (unique).
    fn email(&self) -> &str;
    /// Stored password hash (Argon2/Bcrypt/Scrypt depending on `PasswordConfig`).
    fn password_hash(&self) -> &str;
    /// Whether the account is enabled. Inactive users cannot log in.
    fn is_active(&self) -> bool;
    /// Read-only admin access.
    fn is_staff(&self) -> bool;
    /// Full admin access — bypasses all permission checks.
    fn is_superuser(&self) -> bool;

    /// Custom roles. Returns an empty Vec by default.
    fn roles(&self) -> Vec<String> {
        vec![]
    }

    /// Checks if the user can access the admin.
    ///
    /// Default: active account + (`is_staff` OR `is_superuser`).
    /// Can be overridden for custom logic.
    fn can_access_admin(&self) -> bool {
        self.is_active() && (self.is_staff() || self.is_superuser())
    }
}
