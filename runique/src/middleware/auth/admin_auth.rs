// ═══════════════════════════════════════════════════════════════
// AdminAuth — Contrat pour la vérification du login admin
// ═══════════════════════════════════════════════════════════════
//
// Ce module ne sait pas qu'un panneau admin existe.
// Il définit uniquement le contrat d'authentification.
// C'est src/admin/ qui dépend d'ici, jamais l'inverse.
// ═══════════════════════════════════════════════════════════════

use sea_orm::DatabaseConnection;

/// Données retournées après une authentification admin réussie
#[derive(Debug, Clone)]
pub struct AdminLoginResult {
    pub user_id: i32,
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub roles: Vec<String>,
}

/// Trait à implémenter pour brancher la vérification du login admin
///
/// Retourne `None` si :
/// - L'utilisateur n'existe pas
/// - Le mot de passe est incorrect
/// - Le compte est inactif
/// - L'utilisateur n'a pas les droits admin
///
/// ## Implémentation rapide avec DefaultAdminAuth :
/// ```rust,ignore
/// use runique::middleware::auth::DefaultAdminAuth;
///
/// .with_admin(|a| a.auth(DefaultAdminAuth::<users::Entity>::new()))
/// ```
#[async_trait::async_trait]
pub trait AdminAuth: Send + Sync + 'static {
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
        db: &DatabaseConnection,
    ) -> Option<AdminLoginResult>;
}
