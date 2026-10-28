//! An admin's own rights: the grid a dirigeant (or the superuser) ticks for
//! him, table by table and column by column. Never set from a form field of
//! the account: only this operation writes them.
use super::refusal::{Refusal, RefusalReason};
use crate::auth::current_user::CurrentUser;
use crate::auth::history::{ActionLog, log_action};
use crate::auth::permissions::groupes_droits;
use crate::auth::permissions::rang::Rang;
use crate::auth::user;
use crate::utils::aliases::ADb;
use crate::utils::constante::resource_key;
use crate::utils::pk::Pk;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter, Set, TransactionTrait,
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
/// row of the grid before and after. A table without any table right keeps
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
        use crate::auth::gate::test_support::fixtures::{db, pk};
        use sea_orm::{ConnectionTrait, Schema};
        let db = db().await;
        let schema = Schema::new(sea_orm::DatabaseBackend::Sqlite);
        for table in [
            schema.create_table_from_entity(crate::auth::permissions::groupe::Entity),
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
        set_admin_rights(&actor, pk(33), &[grant], &db).await.unwrap();

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
}
