//! Admin permissions: groups and rights loaded from the database.
pub mod groupe;
pub mod groupes_droits;
pub mod rang;
pub mod rights;
pub mod users_groupes;
use crate::auth::{permissions::rang::Rang, user};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, JoinType, QueryFilter, QuerySelect,
    RelationTrait,
};

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

impl Permission {
    /// What of `self` the `holder` covers: a right ticked here stays only if
    /// he has it, his full right covering an `_own` one. Nothing is derived:
    /// a full right he lacks is lost, never turned into its `_own`.
    fn within(&self, holder: &Self) -> Self {
        Self {
            resource_key: self.resource_key.clone(),
            can_create: self.can_create && holder.can_create,
            can_read: self.can_read && holder.can_read,
            can_read_own: self.can_read_own && (holder.can_read_own || holder.can_read),
            can_update: self.can_update && holder.can_update,
            can_update_own: self.can_update_own && (holder.can_update_own || holder.can_update),
            can_delete: self.can_delete && holder.can_delete,
            can_delete_own: self.can_delete_own && (holder.can_delete_own || holder.can_delete),
            colonnes: self
                .colonnes
                .iter()
                .filter_map(|c| {
                    let h = holder.colonnes.iter().find(|h| h.colonne == c.colonne)?;
                    Some(ColumnRight {
                        colonne: c.colonne.clone(),
                        can_read: c.can_read && h.can_read,
                        can_create: c.can_create && h.can_create,
                        can_update: c.can_update && h.can_update,
                    })
                })
                .collect(),
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

/// What a group keeps of its `rows` under `holder`, its responsable's own
/// grid: each table's rights `within` his, a table he lacks dropped, a table
/// left with no right dropped with its columns. The rows to write back, as
/// the group's.
pub(crate) fn group_rows_within(
    groupe_id: i32,
    rows: Vec<groupes_droits::Model>,
    holder: &[groupes_droits::Model],
) -> Vec<groupes_droits::ActiveModel> {
    use sea_orm::Set;
    let holder = assemble(holder.iter().cloned());
    let row = |key: &str, colonne: &str| groupes_droits::ActiveModel {
        groupe_id: Set(Some(groupe_id)),
        user_id: Set(None),
        resource_key: Set(key.to_string()),
        colonne: Set(colonne.to_string()),
        ..Default::default()
    };
    let mut kept = Vec::new();
    for perm in assemble(rows) {
        let Some(h) = holder.iter().find(|h| h.resource_key == perm.resource_key) else {
            continue;
        };
        let p = perm.within(h);
        let any = p.can_create
            || p.can_read
            || p.can_read_own
            || p.can_update
            || p.can_update_own
            || p.can_delete
            || p.can_delete_own;
        if !any {
            continue;
        }
        let mut table = row(&p.resource_key, "");
        table.can_create = Set(p.can_create);
        table.can_read = Set(p.can_read);
        table.can_read_own = Set(p.can_read_own);
        table.can_update = Set(p.can_update);
        table.can_update_own = Set(p.can_update_own);
        table.can_delete = Set(p.can_delete);
        table.can_delete_own = Set(p.can_delete_own);
        kept.push(table);
        for c in p
            .colonnes
            .iter()
            .filter(|c| c.can_read || c.can_create || c.can_update)
        {
            let mut column = row(&p.resource_key, &c.colonne);
            column.can_read = Set(c.can_read);
            column.can_create = Set(c.can_create);
            column.can_update = Set(c.can_update);
            column.can_read_own = Set(false);
            column.can_update_own = Set(false);
            column.can_delete = Set(false);
            column.can_delete_own = Set(false);
            kept.push(column);
        }
    }
    kept
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

/// Loads a user's groups with their effective permissions — two queries
/// whatever the number of groups. Only an active group whose responsable is
/// active and admin at least is loaded: any other gives nothing, neither
/// rights, nor Prisme role, nor staff access. An admin responsable caps his
/// groups to his own grid, read at each request; a dirigeant or superuser
/// leaves them whole. A database error grants nothing (no group) and is
/// traced, never swallowed silently.
pub async fn pull_groupes_db<C: ConnectionTrait>(
    db: &C,
    user_id: crate::utils::pk::Pk,
) -> Vec<Groupe> {
    let trace_err = |what: &str, e: &sea_orm::DbErr| {
        tracing::error!(user_id = %user_id, error = %e, "{what} failed — no rights granted for this request");
    };
    let rows: Vec<(i32, String, bool, crate::utils::pk::Pk, bool, bool, bool)> =
        match groupe::Entity::find()
            .select_only()
            .column(groupe::Column::Id)
            .column(groupe::Column::Nom)
            .column(groupe::Column::IsStaff)
            .column(groupe::Column::ResponsableId)
            .column(user::Column::IsSuperuser)
            .column(user::Column::IsDirigeant)
            .column(user::Column::IsAdmin)
            .join(JoinType::InnerJoin, groupe::Relation::UsersGroupes.def())
            .join(JoinType::InnerJoin, groupe::Relation::Responsable.def())
            .filter(users_groupes::Column::UserId.eq(user_id))
            .filter(groupe::Column::IsActive.eq(true))
            .filter(user::Column::IsActive.eq(true))
            .filter(
                Condition::any()
                    .add(user::Column::IsSuperuser.eq(true))
                    .add(user::Column::IsDirigeant.eq(true))
                    .add(user::Column::IsAdmin.eq(true)),
            )
            .into_tuple()
            .all(db)
            .await
        {
            Ok(rows) => rows,
            Err(e) => {
                trace_err("loading groups", &e);
                return Vec::new();
            }
        };
    if rows.is_empty() {
        return Vec::new();
    }

    let admins = rows
        .iter()
        .filter(|r| Rang::of(r.4, r.5, r.6) == Rang::Admin)
        .map(|r| r.3);
    let droits = match groupes_droits::Entity::find()
        .filter(
            Condition::any()
                .add(groupes_droits::Column::GroupeId.is_in(rows.iter().map(|r| r.0)))
                .add(groupes_droits::Column::UserId.is_in(admins)),
        )
        .all(db)
        .await
    {
        Ok(droits) => droits,
        Err(e) => {
            trace_err("loading rights", &e);
            return Vec::new();
        }
    };

    rows.into_iter()
        .map(|(id, nom, is_staff, responsable, su, di, ad)| {
            let mut permissions =
                assemble(droits.iter().filter(|m| m.groupe_id == Some(id)).cloned());
            if Rang::of(su, di, ad) == Rang::Admin {
                let grid = assemble(
                    droits
                        .iter()
                        .filter(|m| m.user_id == Some(responsable))
                        .cloned(),
                );
                permissions = permissions
                    .iter()
                    .filter_map(|p| {
                        let h = grid.iter().find(|h| h.resource_key == p.resource_key)?;
                        Some(p.within(h))
                    })
                    .collect();
            }
            Groupe {
                id,
                nom,
                is_staff,
                permissions,
            }
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn perm(set: impl FnOnce(&mut Permission)) -> Permission {
        let mut p = Permission::zeroed("articles".into());
        set(&mut p);
        p
    }

    fn col(name: &str, read: bool, create: bool, update: bool) -> ColumnRight {
        ColumnRight {
            colonne: name.into(),
            can_read: read,
            can_create: create,
            can_update: update,
        }
    }

    #[test]
    fn a_full_right_stays_only_if_the_holder_has_it() {
        let group = perm(|p| {
            p.can_create = true;
            p.can_read = true;
            p.can_update = true;
            p.can_delete = true;
        });
        let all = group.within(&group.clone());
        assert!(all.can_create && all.can_read && all.can_update && all.can_delete);

        let none = group.within(&Permission::zeroed("articles".into()));
        assert!(!none.can_create && !none.can_read && !none.can_update && !none.can_delete);
    }

    #[test]
    fn a_full_right_the_holder_lacks_never_becomes_its_own() {
        let group = perm(|p| {
            p.can_read = true;
            p.can_update = true;
            p.can_delete = true;
        });
        let holder = perm(|p| {
            p.can_read_own = true;
            p.can_update_own = true;
            p.can_delete_own = true;
        });
        let got = group.within(&holder);
        assert!(!got.can_read && !got.can_update && !got.can_delete);
        assert!(!got.can_read_own && !got.can_update_own && !got.can_delete_own);
    }

    #[test]
    fn an_own_right_is_covered_by_the_holders_own_or_full_right() {
        let group = perm(|p| {
            p.can_read_own = true;
            p.can_update_own = true;
            p.can_delete_own = true;
        });
        let own = group.within(&group.clone());
        assert!(own.can_read_own && own.can_update_own && own.can_delete_own);

        let full = group.within(&perm(|p| {
            p.can_read = true;
            p.can_update = true;
            p.can_delete = true;
        }));
        assert!(full.can_read_own && full.can_update_own && full.can_delete_own);
        assert!(!full.can_read && !full.can_update && !full.can_delete);

        let none = group.within(&Permission::zeroed("articles".into()));
        assert!(!none.can_read_own && !none.can_update_own && !none.can_delete_own);
    }

    #[test]
    fn each_own_right_is_covered_by_its_own_full_right_only() {
        let group = perm(|p| {
            p.can_read_own = true;
            p.can_update_own = true;
            p.can_delete_own = true;
        });
        let only_delete = group.within(&perm(|p| p.can_delete = true));
        assert!(only_delete.can_delete_own);
        assert!(!only_delete.can_update_own, "delete never covers update");
        assert!(!only_delete.can_read_own, "delete never covers read");

        let only_update = group.within(&perm(|p| p.can_update = true));
        assert!(only_update.can_update_own);
        assert!(!only_update.can_delete_own && !only_update.can_read_own);
    }

    #[test]
    fn a_column_stays_only_if_the_holder_has_it_with_each_right_anded() {
        let group = perm(|p| {
            p.colonnes = vec![
                col("titre", true, true, true),
                col("prix", true, true, true),
                col("resume", true, true, true),
                col("note", false, false, false),
            ];
        });
        let holder = perm(|p| {
            p.colonnes = vec![
                col("titre", true, true, true),
                col("resume", false, false, false),
                col("note", true, true, true),
            ];
        });
        assert_eq!(
            group.within(&holder).colonnes,
            vec![
                col("titre", true, true, true),
                col("resume", false, false, false),
                col("note", false, false, false),
            ]
        );
    }
}
