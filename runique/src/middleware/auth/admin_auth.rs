//! Trait `AdminAuth` : contrat d'authentification pour le panneau d'administration.
use crate::utils::pk::Pk;
use sea_orm::DatabaseConnection;

/// Données retournées après une authentification admin réussie
#[derive(Debug, Clone)]
pub struct AdminLoginResult {
    pub user_id: Pk,
    pub username: String,
    pub is_staff: bool,
    pub is_superuser: bool,
}

/// Trait à implémenter pour brancher la vérification du login admin
///
/// Retourne `None` si :
/// - L'utilisateur n'existe pas
/// - Le mot de passe est incorrect
/// - Le compte est inactif
/// - L'utilisateur n'a pas les droits admin
///
/// ## Implémentation rapide avec `DefaultAdminAuth` :
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
