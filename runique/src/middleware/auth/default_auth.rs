// ═══════════════════════════════════════════════════════════════
// UserEntity + DefaultAdminAuth
// ═══════════════════════════════════════════════════════════════
//
// UserEntity : côté base de données — comment trouver un user.
// DefaultAdminAuth<E> : implémente AdminAuth automatiquement
//   pour toute entité qui implémente UserEntity.
//
// Usage avec le User built-in de Runique :
//   .auth(DefaultAdminAuth::<BuiltinUser>::new())
//
// Usage avec un modèle custom :
//   impl UserEntity for MyUserEntity { ... }
//   .auth(DefaultAdminAuth::<MyUserEntity>::new())
// ═══════════════════════════════════════════════════════════════

use std::marker::PhantomData;

use sea_orm::DatabaseConnection;

use crate::forms::fields::text::TextField;
use crate::middleware::auth::RuniqueUser;
use crate::middleware::auth::{AdminAuth, AdminLoginResult};

/// Trait côté base de données : comment récupérer un user par username.
///
/// ```rust,ignore
/// impl UserEntity for users::Entity {
///     type Model = users::Model;
///
///     async fn find_by_username(
///         db: &DatabaseConnection,
///         username: &str,
///     ) -> Option<Self::Model> {
///         users::Entity::find()
///             .filter(users::Column::Username.eq(username))
///             .one(db)
///             .await
///             .ok()
///             .flatten()
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait UserEntity: Send + Sync + 'static {
    /// Le modèle retourné par la requête (doit implémenter `RuniqueUser`)
    type Model: RuniqueUser;

    /// Recherche un utilisateur par username en base
    async fn find_by_username(db: &DatabaseConnection, username: &str) -> Option<Self::Model>;
    /// Recherche un utilisateur par email en base
    async fn find_by_email(db: &DatabaseConnection, email: &str) -> Option<Self::Model>;
}

// ═══════════════════════════════════════════════════════════════
// DefaultAdminAuth<E>
// ═══════════════════════════════════════════════════════════════
//
// Implémente AdminAuth automatiquement pour toute entité E
// qui implémente UserEntity.
//
// La logique est toujours la même :
//   1. find_by_username → None = credentials invalides
//   2. can_access_admin → false = droits insuffisants
//   3. verify_password → false = mot de passe incorrect
//   4. Ok → AdminLoginResult
//
// Le dev n'a pas besoin d'écrire cette logique lui-même.
// ═══════════════════════════════════════════════════════════════

/// Adaptateur générique qui transforme n'importe quelle entité
/// implémentant `UserEntity` en `AdminAuth`.
pub struct DefaultAdminAuth<E: UserEntity>(PhantomData<E>);

impl<E: UserEntity> DefaultAdminAuth<E> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<E: UserEntity> Default for DefaultAdminAuth<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl<E: UserEntity> AdminAuth for DefaultAdminAuth<E> {
    async fn authenticate(
        &self,
        username: &str,
        password: &str,
        db: &DatabaseConnection,
    ) -> Option<AdminLoginResult> {
        // 1. Récupérer l'utilisateur depuis la DB
        let user = E::find_by_username(db, username).await?;

        // 2. Vérifier les droits d'accès admin + compte actif
        if !user.can_access_admin() {
            return None;
        }

        // 3. Vérifier le mot de passe (Argon2)
        if !TextField::verify_password(password, user.password_hash()) {
            return None;
        }

        // 4. Tout est bon — retourner les infos de session
        Some(AdminLoginResult {
            user_id: user.user_id(),
            username: user.username().to_string(),
            is_staff: user.is_staff(),
            is_superuser: user.is_superuser(),
            roles: user.roles(),
        })
    }
}
