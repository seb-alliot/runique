use sea_orm_migration::MigrationTrait;

use crate::migration::user;

/// Retourne toutes les migrations internes de Runique.
///
/// ```rust,ignore
/// impl MigratorTrait for Migrator {
///     fn migrations() -> Vec<Box<dyn MigrationTrait>> {
///         let mut m = runique::migration::builtin_migrations();
///         m.extend(vec![
///             Box::new(m20260118_create_users::Migration),
///         ]);
///         m
///     }
/// }
/// ```
pub fn builtin_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(user::Migration) as Box<dyn MigrationTrait>,
        // futures migrations
    ]
}
