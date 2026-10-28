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
        self.create
            || self.read
            || self.read_own
            || self.update
            || self.update_own
            || self.delete
            || self.delete_own
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
/// row. A table without any table right keeps no row: its column rows would
/// open nothing (table right AND column right). The caller has checked every
/// key and column against the admin's models: `grants` names nothing else.
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

    groupes_droits::Entity::delete_many()
        .filter(groupes_droits::Column::UserId.eq(target))
        .exec(&txn)
        .await?;
    let mut tables = Vec::new();
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
        tables.push(grant.resource_key.clone());
    }

    let summary = serde_json::json!({ "droits": { "old": "", "new": tables.join(", ") } });
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
