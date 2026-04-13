//! Admin permissions: groups and rights loaded from the database.
pub mod groupe;
pub mod groupes_droits;
pub mod users_groupes;

use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

// ─────────────────────────────────────────────────────────────────────────────
// Memory structures
// ─────────────────────────────────────────────────────────────────────────────

/// Group permissions on a resource, cached.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    pub resource_key: String,
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub can_update_own: bool,
    pub can_delete_own: bool,
}

/// Group (includes its permissions per resource).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Groupe {
    pub id: crate::utils::pk::Pk,
    pub nom: String,
    pub permissions: Vec<Permission>,
}

// ─────────────────────────────────────────────────────────────────────────────
// DB loading functions — called at login
// ─────────────────────────────────────────────────────────────────────────────

/// Refreshes the memory cache of permissions for a given user.
pub async fn refresh_cache_for_user<C: ConnectionTrait>(db: &C, user_id: crate::utils::pk::Pk) {
    use crate::auth::guard::cache_permissions;
    let groupes = pull_groupes_db(db, user_id).await;
    cache_permissions(user_id, groupes);
}

/// Loads a user's groups with their permissions from the DB.
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

        let droits = groupes_droits::Entity::find()
            .filter(groupes_droits::Column::GroupeId.eq(g.id))
            .all(db)
            .await
            .unwrap_or_default();

        let permissions = droits
            .into_iter()
            .map(|m| Permission {
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
