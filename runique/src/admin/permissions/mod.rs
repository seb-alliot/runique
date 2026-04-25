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

impl Permission {
    /// Creates a zeroed permission entry for a given resource key.
    pub fn zeroed(resource_key: String) -> Self {
        Self {
            resource_key,
            can_create: false,
            can_read: false,
            can_update: false,
            can_delete: false,
            can_update_own: false,
            can_delete_own: false,
        }
    }

    /// Merges `other` into `self` with logical OR on every CRUD flag.
    pub fn merge_from(&mut self, other: &Self) {
        self.can_create |= other.can_create;
        self.can_read |= other.can_read;
        self.can_update |= other.can_update;
        self.can_delete |= other.can_delete;
        self.can_update_own |= other.can_update_own;
        self.can_delete_own |= other.can_delete_own;
    }
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
