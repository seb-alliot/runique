//! Admin permissions: groups and rights loaded from the database.
pub mod groupe;
pub mod groupes_droits;
pub mod users_groupes;

use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

// ─────────────────────────────────────────────────────────────────────────────
// Memory structures
// ─────────────────────────────────────────────────────────────────────────────

/// A group's permissions on a resource, as read from the database.
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
    // Toujours INTEGER, indépendant de `Pk` (big-pk/pk-uuid) — cf.
    // groupe.rs::Model::id / migrations_table.rs::create_eihwaz_groupes_table.
    pub id: i32,
    pub nom: String,
    pub permissions: Vec<Permission>,
}

// ─────────────────────────────────────────────────────────────────────────────
// DB loading functions — called at login
// ─────────────────────────────────────────────────────────────────────────────

/// Deletes rights whose `resource_key` no longer maps to a registered admin
/// resource, closing the referential-integrity gap of the soft `resource_key`
/// string (it points at a code-registry key, not a DB table, so the DB can't
/// enforce it). Without this, a resource removed then replaced by a different one
/// **reusing the same key** would silently inherit the old group's grants — a
/// stealthy privilege escalation. Resources are static per process, so running
/// this once at boot fully closes the window. Returns the number of pruned rows.
///
/// `valid_keys` empty → no-op (never prune everything on a mis-wired registry).
pub async fn prune_orphan_droits<C: ConnectionTrait>(
    db: &C,
    valid_keys: &[&str],
) -> Result<u64, sea_orm::DbErr> {
    if valid_keys.is_empty() {
        return Ok(0);
    }
    let res = groupes_droits::Entity::delete_many()
        .filter(
            groupes_droits::Column::ResourceKey.is_not_in(valid_keys.iter().map(|k| k.to_string())),
        )
        .exec(db)
        .await?;
    Ok(res.rows_affected)
}

/// Loads a user's groups with their permissions from the DB — two queries
/// whatever the number of groups. A database error grants nothing (no group)
/// and is traced, never swallowed silently.
pub async fn pull_groupes_db<C: ConnectionTrait>(
    db: &C,
    user_id: crate::utils::pk::Pk,
) -> Vec<Groupe> {
    let trace_err = |what: &str, e: &sea_orm::DbErr| {
        tracing::error!(user_id = %user_id, error = %e, "{what} failed — no rights granted for this request");
    };
    let rows = match users_groupes::Entity::find()
        .filter(users_groupes::Column::UserId.eq(user_id))
        .find_also_related(groupe::Entity)
        .all(db)
        .await
    {
        Ok(rows) => rows,
        Err(e) => {
            trace_err("loading groups", &e);
            return Vec::new();
        }
    };
    let mut groupes: Vec<Groupe> = rows
        .into_iter()
        .filter_map(|(_, g)| g)
        .map(|g| Groupe {
            id: g.id,
            nom: g.nom,
            permissions: Vec::new(),
        })
        .collect();
    if groupes.is_empty() {
        return groupes;
    }

    let droits = match groupes_droits::Entity::find()
        .filter(groupes_droits::Column::GroupeId.is_in(groupes.iter().map(|g| g.id)))
        .all(db)
        .await
    {
        Ok(droits) => droits,
        Err(e) => {
            trace_err("loading rights", &e);
            return Vec::new();
        }
    };
    for m in droits {
        if let Some(g) = groupes.iter_mut().find(|g| g.id == m.groupe_id) {
            g.permissions.push(Permission {
                resource_key: m.resource_key,
                can_create: m.can_create,
                can_read: m.can_read,
                can_update: m.can_update,
                can_delete: m.can_delete,
                can_update_own: m.can_update_own,
                can_delete_own: m.can_delete_own,
            });
        }
    }
    groupes
}
