//! Admin permissions: groups and rights loaded from the database.
pub mod groupe;
pub mod groupes_droits;
pub mod rang;
pub mod rights;
pub mod users_groupes;

use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

// ─────────────────────────────────────────────────────────────────────────────
// Memory structures
// ─────────────────────────────────────────────────────────────────────────────

/// One column's rights, under its table's right: reading, creating, updating
/// that column. A column without its row is neither read nor written.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct ColumnRight {
    pub colonne: String,
    pub can_read: bool,
    pub can_create: bool,
    pub can_update: bool,
}

/// A group's permissions on a resource, as read from the database.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    pub resource_key: String,
    pub can_create: bool,
    pub can_read: bool,
    #[serde(default)]
    pub can_read_own: bool,
    pub can_update: bool,
    pub can_delete: bool,
    pub can_update_own: bool,
    pub can_delete_own: bool,
    /// The columns this table right reaches: only those listed, never one
    /// added later until it is set.
    #[serde(default)]
    pub colonnes: Vec<ColumnRight>,
}

impl Permission {
    /// Creates a zeroed permission entry for a given resource key.
    pub fn zeroed(resource_key: String) -> Self {
        Self {
            resource_key,
            can_create: false,
            can_read: false,
            can_read_own: false,
            can_update: false,
            can_delete: false,
            can_update_own: false,
            can_delete_own: false,
            colonnes: Vec::new(),
        }
    }

    /// Merges `other` into `self` with logical OR on every CRUD flag.
    pub fn merge_from(&mut self, other: &Self) {
        self.can_create |= other.can_create;
        self.can_read |= other.can_read;
        self.can_read_own |= other.can_read_own;
        self.can_update |= other.can_update;
        self.can_delete |= other.can_delete;
        self.can_update_own |= other.can_update_own;
        self.can_delete_own |= other.can_delete_own;
        for col in &other.colonnes {
            match self.colonnes.iter_mut().find(|c| c.colonne == col.colonne) {
                Some(mine) => {
                    mine.can_read |= col.can_read;
                    mine.can_create |= col.can_create;
                    mine.can_update |= col.can_update;
                }
                None => self.colonnes.push(col.clone()),
            }
        }
    }
}

/// One `Permission` per table from one holder's rows: the table row
/// (`colonne = ''`) gives the table rights, each column row joins that
/// table's `colonnes`. Column rows without a table row give nothing: the
/// table rights stay false, and a column is only read under its table right.
fn assemble(rows: impl IntoIterator<Item = groupes_droits::Model>) -> Vec<Permission> {
    let mut perms: Vec<Permission> = Vec::new();
    for m in rows {
        let i = match perms.iter().position(|p| p.resource_key == m.resource_key) {
            Some(i) => i,
            None => {
                perms.push(Permission::zeroed(m.resource_key.clone()));
                perms.len() - 1
            }
        };
        let p = &mut perms[i];
        if m.colonne.is_empty() {
            p.can_create = m.can_create;
            p.can_read = m.can_read;
            p.can_read_own = m.can_read_own;
            p.can_update = m.can_update;
            p.can_update_own = m.can_update_own;
            p.can_delete = m.can_delete;
            p.can_delete_own = m.can_delete_own;
        } else {
            p.colonnes.push(ColumnRight {
                colonne: m.colonne,
                can_read: m.can_read,
                can_create: m.can_create,
                can_update: m.can_update,
            });
        }
    }
    perms
}

/// Group (includes its permissions per resource).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Groupe {
    // Toujours INTEGER, indépendant de `Pk` (big-pk/pk-uuid) — cf.
    // groupe.rs::Model::id / auth/migrations_table.rs::create_eihwaz_groupes_table.
    pub id: i32,
    pub nom: String,
    pub is_staff: bool,
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
            is_staff: g.is_staff,
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
    for g in &mut groupes {
        g.permissions = assemble(droits.iter().filter(|m| m.groupe_id == Some(g.id)).cloned());
    }
    groupes
}

/// Loads what [`rights_on`](crate::auth::current_user::CurrentUser::rights_on)
/// reads, by the rank: the groups for everyone (a lambda's rights, the staff
/// summary, Prisme's roles), the account's own grid for an admin. The one
/// loader of the admin and of the developer's views alike.
pub async fn load_rights<C: ConnectionTrait>(
    db: &C,
    user: &mut crate::auth::current_user::CurrentUser,
) {
    user.groupes = pull_groupes_db(db, user.id).await;
    user.droits = if user.rang == crate::auth::permissions::rang::Rang::Admin {
        pull_droits_compte(db, user.id).await
    } else {
        Vec::new()
    };
}

/// Loads an admin account's own rights (its `user_id` rows, table and
/// columns), set by the dirigeant. One query, admins only. A database error
/// grants nothing and is traced, never swallowed.
pub async fn pull_droits_compte<C: ConnectionTrait>(
    db: &C,
    user_id: crate::utils::pk::Pk,
) -> Vec<Permission> {
    match groupes_droits::Entity::find()
        .filter(groupes_droits::Column::UserId.eq(user_id))
        .all(db)
        .await
    {
        Ok(rows) => assemble(rows),
        Err(e) => {
            tracing::error!(user_id = %user_id, error = %e, "loading account rights failed — no rights granted for this request");
            Vec::new()
        }
    }
}
