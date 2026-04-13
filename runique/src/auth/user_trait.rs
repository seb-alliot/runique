//! Trait `RuniqueUser` : contrat minimal pour tout modèle utilisateur Runique.

/// Contrat à implémenter sur tout modèle utilisateur pour l'intégration avec le framework.
pub trait RuniqueUser: Send + Sync {
    fn user_id(&self) -> crate::utils::pk::Pk;
    fn username(&self) -> &str;
    fn email(&self) -> &str;
    fn password_hash(&self) -> &str;
    fn is_active(&self) -> bool;
    fn is_staff(&self) -> bool;
    fn is_superuser(&self) -> bool;

    /// Rôles personnalisés. Retourne un Vec vide par défaut.
    fn roles(&self) -> Vec<String> {
        vec![]
    }

    /// Vérifie si l'utilisateur peut accéder à l'admin.
    ///
    /// Par défaut : compte actif + (`is_staff` OU `is_superuser`).
    /// Peut être surchargé pour une logique custom.
    fn can_access_admin(&self) -> bool {
        self.is_active() && (self.is_staff() || self.is_superuser())
    }
}
