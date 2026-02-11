// ═══════════════════════════════════════════════════════════════
// RuniqueUser — Contrat pour tout modèle utilisateur
// ═══════════════════════════════════════════════════════════════
//
// Implémentation minimale requise :
//
//   impl RuniqueUser for MyUserModel {
//       fn user_id(&self) -> i32        { self.id }
//       fn username(&self) -> &str      { &self.username }
//       fn password_hash(&self) -> &str { &self.password }
//       fn is_active(&self) -> bool     { self.is_active }
//       fn is_staff(&self) -> bool      { self.is_staff }
//       fn is_superuser(&self) -> bool  { self.is_superuser }
//   }
// ═══════════════════════════════════════════════════════════════

pub trait RuniqueUser: Send + Sync {
    fn user_id(&self) -> i32;
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
    /// Par défaut : compte actif + (is_staff OU is_superuser).
    /// Peut être surchargé pour une logique custom.
    fn can_access_admin(&self) -> bool {
        self.is_active() && (self.is_staff() || self.is_superuser())
    }
}
