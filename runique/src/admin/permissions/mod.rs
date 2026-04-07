//! Permissions admin : permissions CRUD, groupes chargés depuis la base.
pub mod droit;
pub mod groupe;
pub mod users_groupes;

use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

// ─────────────────────────────────────────────────────────────────────────────
// Structures Mémorielles
// ─────────────────────────────────────────────────────────────────────────────

/// Matrice CRUD (Permission) stockée en cache (anciennement Droit)
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    pub id: crate::utils::pk::Pk,
    pub resource_key: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub can_update_own: bool,
    pub can_delete_own: bool,
}

/// Groupe (inclut ses permissions).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Groupe {
    pub id: crate::utils::pk::Pk,
    pub nom: String,
    pub permissions: Vec<Permission>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Fonctions de chargement DB — appelées au login
// ─────────────────────────────────────────────────────────────────────────────

/// Rafraîchit le cache mémoire des permissions pour un utilisateur donné.
/// Appelé par les signaux SeaORM après toute modification des groupes.
pub async fn refresh_cache_for_user<C: ConnectionTrait>(db: &C, user_id: crate::utils::pk::Pk) {
    use crate::middleware::auth::permissions_cache::cache_permissions;
    let groupes = pull_groupes_db(db, user_id).await;
    cache_permissions(user_id, groupes);
}

/// Charge les groupes d'un utilisateur avec leurs permissions (matrice CRUD) depuis la DB.
pub async fn pull_groupes_db<C: ConnectionTrait>(
    db: &C,
    user_id: crate::utils::pk::Pk,
) -> Vec<Groupe> {
    let groupe_rows = users_groupes::Entity::find()
        .filter(users_groupes::Column::UserId.eq(user_id))
        .find_also_related(groupe::Entity)
        .all(db)
        .await
        .unwrap_or_default();

    let mut groupes = Vec::new();

    for (_, maybe_groupe) in groupe_rows {
        let Some(g) = maybe_groupe else { continue };

        // Charge les droits (Permissions) directement liés au groupe (relation 1-N)
        let droits = droit::Entity::find()
            .filter(droit::Column::GroupeId.eq(g.id))
            .all(db)
            .await
            .unwrap_or_default();

        let permissions = droits
            .into_iter()
            .map(|m| Permission {
                id: m.id,
                resource_key: m.resource_key,
                can_create: m.can_create,
                can_read: m.can_read,
                can_update: m.can_update,
                can_delete: m.can_delete,
                can_update_own: m.can_update_own,
                can_delete_own: m.can_delete_own,
            })
            .collect();

        groupes.push(Groupe {
            id: g.id,
            nom: g.nom,
            permissions,
        });
    }

    groupes
}
