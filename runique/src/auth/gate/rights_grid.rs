//! An admin's own rights: the grid a dirigeant (or the superuser) ticks for
//! him, table by table and column by column. Never set from a form field of
//! the account: only this operation writes them.
use super::refusal::{Refusal, RefusalReason};
use crate::auth::current_user::CurrentUser;
use crate::auth::history::{ActionLog, log_action};
use crate::auth::permissions::rang::Rang;
use crate::auth::permissions::{group_rows_within, groupe, groupes_droits};
use crate::auth::user;
use crate::utils::aliases::ADb;
use crate::utils::constante::resource_key;
use crate::utils::pk::Pk;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, QuerySelect, Set,
    TransactionTrait,
};

/// The table rights of one row: create, read, read_own, update, update_own,
/// delete, delete_own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TableRights {
    pub create: bool,
    pub read: bool,
    pub read_own: bool,
    pub update: bool,
    pub update_own: bool,
    pub delete: bool,
    pub delete_own: bool,
}

impl TableRights {
    fn any(&self) -> bool {
        !self.names().is_empty()
    }

    /// The ticked rights, in the grid's order.
    fn names(&self) -> Vec<&'static str> {
        [
            (self.create, "create"),
            (self.read, "read"),
            (self.read_own, "read_own"),
            (self.update, "update"),
            (self.update_own, "update_own"),
            (self.delete, "delete"),
            (self.delete_own, "delete_own"),
        ]
        .into_iter()
        .filter_map(|(ticked, name)| ticked.then_some(name))
        .collect()
    }
}

/// What a column may: read, create, update.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ColumnGrant {
    pub column: String,
    pub read: bool,
    pub create: bool,
    pub update: bool,
}

/// One table of the grid.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TableGrant {
    pub resource_key: String,
    pub table: TableRights,
    pub columns: Vec<ColumnGrant>,
}

impl ColumnGrant {
    /// The ticked rights, in the grid's order.
    fn names(&self) -> Vec<&'static str> {
        [
            (self.read, "read"),
            (self.create, "create"),
            (self.update, "update"),
        ]
        .into_iter()
        .filter_map(|(ticked, name)| ticked.then_some(name))
        .collect()
    }
}

/// The grid as it is stored: a table's row, then its columns' rows.
fn from_rows(rows: &[groupes_droits::Model]) -> Vec<TableGrant> {
    let mut tables: std::collections::BTreeMap<&str, TableGrant> = Default::default();
    for row in rows {
        let table = tables
            .entry(row.resource_key.as_str())
            .or_insert_with(|| TableGrant {
                resource_key: row.resource_key.clone(),
                ..Default::default()
            });
        if row.colonne.is_empty() {
            table.table = TableRights {
                create: row.can_create,
                read: row.can_read,
                read_own: row.can_read_own,
                update: row.can_update,
                update_own: row.can_update_own,
                delete: row.can_delete,
                delete_own: row.can_delete_own,
            };
        } else {
            table.columns.push(ColumnGrant {
                column: row.colonne.clone(),
                read: row.can_read,
                create: row.can_create,
                update: row.can_update,
            });
        }
    }
    tables.into_values().collect()
}

/// `grants` as the history shows them, tables and columns sorted so the old
/// grid and the new one compare: `notes (create, read; title: read)`. Only
/// what opens something: a table without a table right, a column without a
/// right, aren't written either.
fn describe(grants: &[TableGrant]) -> String {
    let mut tables: Vec<&TableGrant> = grants.iter().filter(|g| g.table.any()).collect();
    tables.sort_by(|a, b| a.resource_key.cmp(&b.resource_key));
    tables
        .into_iter()
        .map(|grant| {
            let mut columns: Vec<&ColumnGrant> = grant
                .columns
                .iter()
                .filter(|c| !c.names().is_empty())
                .collect();
            columns.sort_by(|a, b| a.column.cmp(&b.column));
            let mut parts = vec![grant.table.names().join(", ")];
            parts.extend(
                columns
                    .into_iter()
                    .map(|c| format!("{}: {}", c.column, c.names().join(", "))),
            );
            format!("{} ({})", grant.resource_key, parts.join("; "))
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// The rights an admin may hold on the groups: create, and act on those he
/// answers for. A full right would open the other admins' groups and the
/// dirigeant's: never set, whoever ticks it.
pub(crate) const ADMIN_GROUP_RIGHTS: [&str; 4] = ["create", "read_own", "update_own", "delete_own"];

/// Whether `grant` keeps to `ADMIN_GROUP_RIGHTS` on the groups' table.
fn keeps_group_floor(grant: &TableGrant) -> bool {
    grant.resource_key != resource_key::GROUPES
        || !(grant.table.read || grant.table.update || grant.table.delete)
}

fn refused(reason: RefusalReason) -> DbErr {
    DbErr::Custom(Refusal::new(reason).message())
}

/// May `actor` set `target`'s grid: a dirigeant or the superuser, on an
/// admin, never on themselves. The target's rank is read here, in the write's
/// transaction for `set_admin_rights`.
pub(crate) fn may_set(actor: &CurrentUser, target: &user::Model) -> Result<(), DbErr> {
    if actor.rang < Rang::Dirigeant || target.id == actor.id {
        return Err(refused(RefusalReason::RankTooLow));
    }
    if Rang::from(target) != Rang::Admin {
        return Err(refused(RefusalReason::NotFound));
    }
    Ok(())
}

/// Replaces `target`'s whole grid with `grants`, as `actor`, with a history
/// row of the grid before and after; the groups `target` answers for are cut
/// down to it. A table without any table right keeps
/// no row: its column rows would open nothing (table right AND column
/// right). The caller has checked every key and column against the admin's
/// models: `grants` names nothing else.
pub(crate) async fn set_admin_rights(
    actor: &CurrentUser,
    target: Pk,
    grants: &[TableGrant],
    db: &ADb,
) -> Result<(), DbErr> {
    let txn = db.begin().await?;
    let Some(account) = user::Entity::find_by_id(target).one(&txn).await? else {
        return Err(refused(RefusalReason::NotFound));
    };
    may_set(actor, &account)?;
    if !grants.iter().all(keeps_group_floor) {
        return Err(refused(RefusalReason::NoRight));
    }

    let before = groupes_droits::Entity::find()
        .filter(groupes_droits::Column::UserId.eq(target))
        .all(&txn)
        .await?;
    groupes_droits::Entity::delete_many()
        .filter(groupes_droits::Column::UserId.eq(target))
        .exec(&txn)
        .await?;
    for grant in grants.iter().filter(|g| g.table.any()) {
        let t = grant.table;
        groupes_droits::ActiveModel {
            groupe_id: Set(None),
            user_id: Set(Some(target)),
            resource_key: Set(grant.resource_key.clone()),
            colonne: Set(String::new()),
            can_create: Set(t.create),
            can_read: Set(t.read),
            can_read_own: Set(t.read_own),
            can_update: Set(t.update),
            can_update_own: Set(t.update_own),
            can_delete: Set(t.delete),
            can_delete_own: Set(t.delete_own),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        for col in grant
            .columns
            .iter()
            .filter(|c| c.read || c.create || c.update)
        {
            groupes_droits::ActiveModel {
                groupe_id: Set(None),
                user_id: Set(Some(target)),
                resource_key: Set(grant.resource_key.clone()),
                colonne: Set(col.column.clone()),
                can_create: Set(col.create),
                can_read: Set(col.read),
                can_read_own: Set(false),
                can_update: Set(col.update),
                can_update_own: Set(false),
                can_delete: Set(false),
                can_delete_own: Set(false),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }
    }

    // A right taken from the admin is taken from the groups he answers for,
    // in the same transaction. Stored, not only read through his grid: a
    // right given back to him later wakes no dormant row of theirs.
    let groups: Vec<i32> = groupe::Entity::find()
        .select_only()
        .column(groupe::Column::Id)
        .filter(groupe::Column::ResponsableId.eq(target))
        .into_tuple()
        .all(&txn)
        .await?;
    if !groups.is_empty() {
        let grid = groupes_droits::Entity::find()
            .filter(groupes_droits::Column::UserId.eq(target))
            .all(&txn)
            .await?;
        let rows = groupes_droits::Entity::find()
            .filter(groupes_droits::Column::GroupeId.is_in(groups.iter().copied()))
            .all(&txn)
            .await?;
        groupes_droits::Entity::delete_many()
            .filter(groupes_droits::Column::GroupeId.is_in(groups.iter().copied()))
            .exec(&txn)
            .await?;
        for id in groups {
            let own = rows
                .iter()
                .filter(|r| r.groupe_id == Some(id))
                .cloned()
                .collect();
            let kept = group_rows_within(id, own, &grid);
            if !kept.is_empty() {
                groupes_droits::Entity::insert_many(kept).exec(&txn).await?;
            }
        }
    }

    let summary = serde_json::json!({
        "droits": { "old": describe(&from_rows(&before)), "new": describe(grants) }
    });
    log_action(
        &txn,
        ActionLog {
            user_id: actor.id,
            username: &actor.username,
            resource_key: resource_key::USERS,
            object_pk: &target.to_string(),
            object_repr: &account.username,
            action: "rights",
            summary: Some(summary.to_string()),
            batch_id: None,
        },
    )
    .await?;
    txn.commit().await
}

/// May `actor` set group `group`'s rights: the superuser and a dirigeant on
/// any group; an admin on a group he answers for, when his grid lets him
/// update the groups.
pub(crate) fn may_set_group(actor: &CurrentUser, group: &groupe::Model) -> Result<(), DbErr> {
    if actor.rang >= Rang::Dirigeant {
        return Ok(());
    }
    let delegated = actor.rang == Rang::Admin
        && group.responsable_id == actor.id
        && actor
            .droits
            .iter()
            .any(|p| p.resource_key == resource_key::GROUPES && (p.can_update || p.can_update_own));
    if delegated {
        Ok(())
    } else {
        Err(refused(RefusalReason::NotFound))
    }
}

/// Whether `holder`'s grid holds every right `grant` ticks: an admin gives a
/// group only what he has himself, his full right covering an `_own` one.
fn covered(grant: &TableGrant, holder: &[crate::auth::permissions::Permission]) -> bool {
    let columns_ticked = grant.columns.iter().any(|c| c.read || c.create || c.update);
    if !grant.table.any() && !columns_ticked {
        return true;
    }
    let Some(h) = holder.iter().find(|p| p.resource_key == grant.resource_key) else {
        return false;
    };
    let t = &grant.table;
    let table = (!t.create || h.can_create)
        && (!t.read || h.can_read)
        && (!t.read_own || h.can_read_own || h.can_read)
        && (!t.update || h.can_update)
        && (!t.update_own || h.can_update_own || h.can_update)
        && (!t.delete || h.can_delete)
        && (!t.delete_own || h.can_delete_own || h.can_delete);
    table
        && grant.columns.iter().all(|c| {
            (!c.read && !c.create && !c.update)
                || h.colonnes
                    .iter()
                    .find(|hc| hc.colonne == c.column)
                    .is_some_and(|hc| {
                        (!c.read || hc.can_read)
                            && (!c.create || hc.can_create)
                            && (!c.update || hc.can_update)
                    })
        })
}

/// Replaces group `group_id`'s whole grid with `grants`, as `actor`, with a
/// history row of the grid before and after. Never a framework table (the
/// base refuses it too); from an admin, only rights he holds himself,
/// refused whole otherwise.
pub(crate) async fn set_group_rights(
    actor: &CurrentUser,
    group_id: i32,
    grants: &[TableGrant],
    db: &ADb,
) -> Result<(), DbErr> {
    let txn = db.begin().await?;
    let Some(group) = groupe::Entity::find_by_id(group_id).one(&txn).await? else {
        return Err(refused(RefusalReason::NotFound));
    };
    may_set_group(actor, &group)?;
    let framework = grants
        .iter()
        .any(|g| resource_key::FRAMEWORK_KEYS.contains(&g.resource_key.as_str()));
    let beyond = actor.rang == Rang::Admin && !grants.iter().all(|g| covered(g, &actor.droits));
    if framework || beyond {
        return Err(refused(RefusalReason::NoRight));
    }

    let before = groupes_droits::Entity::find()
        .filter(groupes_droits::Column::GroupeId.eq(group_id))
        .all(&txn)
        .await?;
    groupes_droits::Entity::delete_many()
        .filter(groupes_droits::Column::GroupeId.eq(group_id))
        .exec(&txn)
        .await?;
    let rows = grant_rows(grants, Some(group_id), None);
    if !rows.is_empty() {
        groupes_droits::Entity::insert_many(rows).exec(&txn).await?;
    }

    let summary = serde_json::json!({
        "droits": { "old": describe(&from_rows(&before)), "new": describe(grants) }
    });
    log_action(
        &txn,
        ActionLog {
            user_id: actor.id,
            username: &actor.username,
            resource_key: resource_key::GROUPES,
            object_pk: &group_id.to_string(),
            object_repr: &group.nom,
            action: "rights",
            summary: Some(summary.to_string()),
            batch_id: None,
        },
    )
    .await?;
    txn.commit().await
}

/// The rows of `grants` for one holder, a group or an account: a table's row
/// when it has a table right, then its columns that have one.
fn grant_rows(
    grants: &[TableGrant],
    groupe_id: Option<i32>,
    user_id: Option<Pk>,
) -> Vec<groupes_droits::ActiveModel> {
    let mut rows = Vec::new();
    for grant in grants.iter().filter(|g| g.table.any()) {
        let t = grant.table;
        rows.push(groupes_droits::ActiveModel {
            groupe_id: Set(groupe_id),
            user_id: Set(user_id),
            resource_key: Set(grant.resource_key.clone()),
            colonne: Set(String::new()),
            can_create: Set(t.create),
            can_read: Set(t.read),
            can_read_own: Set(t.read_own),
            can_update: Set(t.update),
            can_update_own: Set(t.update_own),
            can_delete: Set(t.delete),
            can_delete_own: Set(t.delete_own),
            ..Default::default()
        });
        for col in grant
            .columns
            .iter()
            .filter(|c| c.read || c.create || c.update)
        {
            rows.push(groupes_droits::ActiveModel {
                groupe_id: Set(groupe_id),
                user_id: Set(user_id),
                resource_key: Set(grant.resource_key.clone()),
                colonne: Set(col.column.clone()),
                can_create: Set(col.create),
                can_read: Set(col.read),
                can_read_own: Set(false),
                can_update: Set(col.update),
                can_update_own: Set(false),
                can_delete: Set(false),
                can_delete_own: Set(false),
                ..Default::default()
            });
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(
        key: &str,
        colonne: &str,
        read: bool,
        create: bool,
        update: bool,
    ) -> groupes_droits::Model {
        groupes_droits::Model {
            id: 0,
            groupe_id: None,
            user_id: None,
            resource_key: key.into(),
            colonne: colonne.into(),
            can_create: create,
            can_read: read,
            can_read_own: false,
            can_update: update,
            can_update_own: false,
            can_delete: colonne.is_empty() && update,
            can_delete_own: false,
        }
    }

    #[test]
    fn a_stored_grid_reads_back_as_it_was_written() {
        let rows = [
            row("notes", "title", true, false, true),
            row("notes", "", true, true, true),
            row("blog", "", true, false, false),
        ];
        assert_eq!(
            describe(&from_rows(&rows)),
            "blog (read) | notes (create, read, update, delete; title: read, update)"
        );
    }

    #[test]
    fn a_change_inside_a_table_shows() {
        let before = [row("notes", "", true, false, false)];
        let after = [row("notes", "", true, false, true)];
        assert_ne!(describe(&from_rows(&before)), describe(&from_rows(&after)));
    }

    #[test]
    fn what_opens_nothing_is_left_out() {
        let grants = [
            TableGrant {
                resource_key: "ghost".into(),
                table: TableRights::default(),
                columns: vec![ColumnGrant {
                    column: "title".into(),
                    read: true,
                    ..Default::default()
                }],
            },
            TableGrant {
                resource_key: "notes".into(),
                table: TableRights {
                    read: true,
                    ..Default::default()
                },
                columns: vec![ColumnGrant {
                    column: "body".into(),
                    ..Default::default()
                }],
            },
        ];
        assert_eq!(describe(&grants), "notes (read)");
        assert_eq!(describe(&[]), "");
    }

    /// The grid read back from the base is the one sent: each table right on
    /// its own, a column with a single right kept, one without any left out.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn every_right_of_the_grid_is_written() {
        use crate::auth::gate::test_support::fixtures::pk;
        let (db, actor) = grid_db().await;
        let column = |name: &str, read, create, update| ColumnGrant {
            column: name.into(),
            read,
            create,
            update,
        };
        let grant = TableGrant {
            resource_key: "notes".into(),
            table: TableRights {
                create: true,
                read: true,
                read_own: true,
                update: true,
                update_own: true,
                delete: true,
                delete_own: true,
            },
            columns: vec![
                column("a", false, true, false),
                column("b", false, false, true),
                column("c", true, false, false),
                column("d", false, false, false),
            ],
        };
        set_admin_rights(&actor, pk(33), &[grant], &db)
            .await
            .unwrap();

        let rows = groupes_droits::Entity::find()
            .filter(groupes_droits::Column::UserId.eq(pk(33)))
            .all(&db)
            .await
            .unwrap();
        assert_eq!(
            describe(&from_rows(&rows)),
            "notes (create, read, read_own, update, update_own, delete, delete_own; \
             a: create; b: update; c: read)"
        );
        assert_eq!(rows.len(), 4, "the table's row and three columns, not `d`");
    }

    /// The gate's fixture base with the groups' tables, and the dirigeant
    /// `boss` (32) who sets the grid of the admin `delegate` (33).
    #[cfg(feature = "sqlite")]
    async fn grid_db() -> (ADb, CurrentUser) {
        use crate::auth::gate::test_support::fixtures::{db, pk};
        use sea_orm::{ConnectionTrait, Schema};
        let db = db().await;
        let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
        for table in [
            schema.create_table_from_entity(groupe::Entity),
            schema.create_table_from_entity(groupes_droits::Entity),
        ] {
            db.execute(&table).await.unwrap();
        }
        let actor = CurrentUser {
            id: pk(32),
            username: "boss".into(),
            rang: Rang::Dirigeant,
            groupes: Vec::new(),
            droits: Vec::new(),
        };
        (db, actor)
    }

    /// The rows of group `id`, as the grid reads them.
    #[cfg(feature = "sqlite")]
    async fn group_grid(db: &ADb, id: i32) -> String {
        let rows = groupes_droits::Entity::find()
            .filter(groupes_droits::Column::GroupeId.eq(id))
            .all(db)
            .await
            .unwrap();
        describe(&from_rows(&rows))
    }

    /// Taking a right from an admin takes it from the groups he answers for,
    /// stored; a table he no longer has leaves them. A group answering to
    /// someone else keeps all of its rights.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn a_right_taken_from_an_admin_is_taken_from_his_groups() {
        use crate::auth::gate::test_support::fixtures::pk;
        let (db, actor) = grid_db().await;
        for (id, responsable) in [(1, pk(33)), (2, pk(32))] {
            groupe::ActiveModel {
                id: Set(id),
                nom: Set(format!("g{id}")),
                is_staff: Set(false),
                responsable_id: Set(responsable),
                repris_par: Set(None),
                is_active: Set(true),
            }
            .insert(&db)
            .await
            .unwrap();
            for (key, colonne) in [("notes", ""), ("notes", "a"), ("blog", "")] {
                groupes_droits::ActiveModel {
                    groupe_id: Set(Some(id)),
                    user_id: Set(None),
                    resource_key: Set(key.into()),
                    colonne: Set(colonne.into()),
                    can_create: Set(true),
                    can_read: Set(true),
                    can_read_own: Set(false),
                    can_update: Set(true),
                    can_update_own: Set(false),
                    can_delete: Set(false),
                    can_delete_own: Set(false),
                    ..Default::default()
                }
                .insert(&db)
                .await
                .unwrap();
            }
        }
        let before = group_grid(&db, 1).await;

        // The admin keeps reading the notes and their column `a`, nothing else.
        let grant = TableGrant {
            resource_key: "notes".into(),
            table: TableRights {
                read: true,
                ..Default::default()
            },
            columns: vec![ColumnGrant {
                column: "a".into(),
                read: true,
                ..Default::default()
            }],
        };
        set_admin_rights(&actor, pk(33), &[grant], &db)
            .await
            .unwrap();

        assert_eq!(group_grid(&db, 1).await, "notes (read; a: read)");
        assert_eq!(
            group_grid(&db, 2).await,
            before,
            "another's group untouched"
        );
    }

    /// The gate keeps the groups' floor even when the page is bypassed: a
    /// full right on the groups, for an admin, is refused whole.
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn an_admin_never_gets_a_full_right_on_the_groups() {
        use crate::auth::gate::test_support::fixtures::pk;
        let (db, actor) = grid_db().await;
        let grant = |table: TableRights| TableGrant {
            resource_key: resource_key::GROUPES.into(),
            table,
            columns: Vec::new(),
        };
        let full = grant(TableRights {
            read: true,
            ..Default::default()
        });
        assert!(set_admin_rights(&actor, pk(33), &[full], &db).await.is_err());
        let floor = grant(TableRights {
            create: true,
            read_own: true,
            update_own: true,
            delete_own: true,
            ..Default::default()
        });
        assert!(set_admin_rights(&actor, pk(33), &[floor], &db).await.is_ok());
    }
}
