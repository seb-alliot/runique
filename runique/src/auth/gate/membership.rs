//! An account's groups: the ones the actor manages it may join or leave.
//! Never a form field of the account: only this operation writes them.
use super::refusal::{Refusal, RefusalReason};
use super::rights_grid::may_set_group;
use crate::auth::current_user::CurrentUser;
use crate::auth::history::{ActionLog, log_action};
use crate::auth::permissions::rang::Rang;
use crate::auth::permissions::{groupe, users_groupes};
use crate::auth::user;
use crate::utils::aliases::ADb;
use crate::utils::constante::resource_key;
use crate::utils::pk::Pk;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, QueryFilter, QueryOrder, Set,
    TransactionTrait,
};
use std::collections::HashSet;

fn refused(reason: RefusalReason) -> DbErr {
    DbErr::Custom(Refusal::new(reason).message())
}

/// The groups `actor` manages, by name: every one for the superuser and a
/// dirigeant, those he answers for and may set for a delegated admin.
pub(crate) async fn managed_groups<C: ConnectionTrait>(
    actor: &CurrentUser,
    db: &C,
) -> Result<Vec<groupe::Model>, DbErr> {
    let query = groupe::Entity::find().order_by_asc(groupe::Column::Nom);
    let groups = if actor.rang >= Rang::Dirigeant {
        query.all(db).await?
    } else {
        query
            .filter(groupe::Column::ResponsableId.eq(actor.id))
            .all(db)
            .await?
    };
    Ok(groups
        .into_iter()
        .filter(|g| may_set_group(actor, g).is_ok())
        .collect())
}

/// Whether `actor` may set `target`'s groups: an account of a rank below
/// his, never his own.
pub(crate) fn may_set_memberships(actor: &CurrentUser, target: &user::Model) -> bool {
    target.id != actor.id && Rang::from(target) < actor.rang
}

/// Sets which of the groups `actor` manages `target` belongs to: `wanted`,
/// the others left as they were. Refused whole when `wanted` names a group
/// he doesn't manage, or puts an account with a rank in a group an admin
/// answers for (such a group holds site accounts only). One history row.
pub(crate) async fn set_memberships(
    actor: &CurrentUser,
    target: Pk,
    wanted: &[i32],
    db: &ADb,
) -> Result<(), DbErr> {
    let txn = db.begin().await?;
    let Some(account) = user::Entity::find_by_id(target).one(&txn).await? else {
        return Err(refused(RefusalReason::NotFound));
    };
    if !may_set_memberships(actor, &account) {
        return Err(refused(RefusalReason::NotFound));
    }
    let managed = managed_groups(actor, &txn).await?;
    let managed_ids: HashSet<i32> = managed.iter().map(|g| g.id).collect();
    let wanted: HashSet<i32> = wanted.iter().copied().collect();
    if !wanted.is_subset(&managed_ids) {
        return Err(refused(RefusalReason::NoRight));
    }
    if Rang::from(&account) != Rang::Lambda {
        let responsables: Vec<Pk> = managed
            .iter()
            .filter(|g| wanted.contains(&g.id))
            .map(|g| g.responsable_id)
            .collect();
        let admin_answers = user::Entity::find()
            .filter(user::Column::Id.is_in(responsables))
            .filter(user::Column::IsAdmin.eq(true))
            .one(&txn)
            .await?
            .is_some();
        if admin_answers {
            return Err(refused(RefusalReason::NoRight));
        }
    }

    let current: HashSet<i32> = users_groupes::Entity::find()
        .filter(users_groupes::Column::UserId.eq(target))
        .filter(users_groupes::Column::GroupeId.is_in(managed_ids.iter().copied()))
        .all(&txn)
        .await?
        .into_iter()
        .map(|m| m.groupe_id)
        .collect();
    let leaving: Vec<i32> = current.difference(&wanted).copied().collect();
    if !leaving.is_empty() {
        users_groupes::Entity::delete_many()
            .filter(users_groupes::Column::UserId.eq(target))
            .filter(users_groupes::Column::GroupeId.is_in(leaving))
            .exec(&txn)
            .await?;
    }
    let joining: Vec<users_groupes::ActiveModel> = wanted
        .difference(&current)
        .map(|id| users_groupes::ActiveModel {
            user_id: Set(target),
            groupe_id: Set(*id),
        })
        .collect();
    if !joining.is_empty() {
        users_groupes::Entity::insert_many(joining)
            .exec(&txn)
            .await?;
    }

    let names = |ids: &HashSet<i32>| {
        managed
            .iter()
            .filter(|g| ids.contains(&g.id))
            .map(|g| g.nom.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let summary = serde_json::json!({
        "groupes": { "old": names(&current), "new": names(&wanted) }
    });
    log_action(
        &txn,
        ActionLog {
            user_id: actor.id,
            username: &actor.username,
            resource_key: resource_key::USERS,
            object_pk: &target.to_string(),
            object_repr: &account.username,
            action: "groups",
            summary: Some(summary.to_string()),
            batch_id: None,
        },
    )
    .await?;
    txn.commit().await
}
