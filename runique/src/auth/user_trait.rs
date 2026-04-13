//! `RuniqueUser` trait: minimal contract for any Runique user model.

/// Contract to implement on any user model for framework integration.
pub trait RuniqueUser: Send + Sync {
    fn user_id(&self) -> crate::utils::pk::Pk;
    fn username(&self) -> &str;
    fn email(&self) -> &str;
    fn password_hash(&self) -> &str;
    fn is_active(&self) -> bool;
    fn is_staff(&self) -> bool;
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
