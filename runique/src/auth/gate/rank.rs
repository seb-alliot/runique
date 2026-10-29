//! A change of rank: never a form field, only this operation. The superuser
//! names dirigeants, a dirigeant names admins; nobody touches their own rank
//! or one equal to theirs, and the admin keeps a dirigeant above them.
use super::refusal::{Refusal, RefusalReason};
use super::save::take_back;
use crate::auth::current_user::CurrentUser;
use crate::auth::history::{ActionLog, log_action};
use crate::auth::permissions::groupes_droits;
use crate::auth::permissions::rang::Rang;
use crate::auth::user::{self, Column};
use crate::utils::aliases::ADb;
use crate::utils::constante::resource_key;
use crate::utils::pk::Pk;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, PaginatorTrait, QueryFilter, TransactionTrait,
};

fn refused(reason: RefusalReason) -> DbErr {
    DbErr::Custom(Refusal::new(reason).message())
}

/// The three flags of `rang`.
fn flags(rang: Rang) -> (bool, bool, bool) {
    match rang {
        Rang::Superuser => (true, false, false),
        Rang::Dirigeant => (false, true, false),
        Rang::Admin => (false, false, true),
        Rang::Lambda => (false, false, false),
    }
}

/// How many accounts outside `ids` hold `col`, active ones only when `active`.
async fn others_with<C: ConnectionTrait>(
    db: &C,
    col: Column,
    ids: &[Pk],
    active: bool,
) -> Result<u64, DbErr> {
    let mut query = user::Entity::find()
        .filter(col.eq(true))
        .filter(Column::Id.is_not_in(ids.iter().copied()));
    if active {
        query = query.filter(Column::IsActive.eq(true));
    }
    query.count(db).await
}

/// Gives the accounts `ids` the rank `to`, as `actor`: all of them or none,
/// with one history row each. Refused, with the gate's one message, when the
/// actor's rank isn't strictly above `to`, when an account is the actor's
/// own, missing or out of reach, or when admins would be left without an
/// active dirigeant. The groups of an account losing rank go to the actor,
/// taken back.
pub(crate) async fn set_rank(
    actor: &CurrentUser,
    ids: &[Pk],
    to: Rang,
    db: &ADb,
) -> Result<u64, DbErr> {
    if actor.rang < Rang::Dirigeant || to >= actor.rang {
        return Err(refused(RefusalReason::RankTooLow));
    }
    if ids.is_empty() || ids.contains(&actor.id) {
        return Err(refused(RefusalReason::NoRight));
    }
    let Some(reach) = actor.rang.cibles_gerables() else {
        return Err(refused(RefusalReason::RankTooLow));
    };

    let txn = db.begin().await?;
    // Every account must exist and be of a rank strictly below the actor's.
    let targets = user::Entity::find()
        .filter(Column::Id.is_in(ids.iter().copied()))
        .filter(reach.clone())
        .all(&txn)
        .await?;
    if targets.len() != ids.len() {
        return Err(refused(RefusalReason::NotFound));
    }

    // Any loss of rank caps a responsable's groups by the lower rank: the
    // actor takes them back first, whole.
    let demoted = targets.iter().filter(|t| Rang::from(*t) > to).map(|t| t.id);
    take_back(&txn, actor, demoted).await?;
    let leaving_admin: Vec<Pk> = targets
        .iter()
        .filter(|t| Rang::from(*t) == Rang::Admin && to != Rang::Admin)
        .map(|t| t.id)
        .collect();

    // No admin without an active dirigeant, once the change is made.
    let given = targets.len() as u64;
    let dirigeants = others_with(&txn, Column::IsDirigeant, ids, true).await?
        + if to == Rang::Dirigeant { given } else { 0 };
    let admins = others_with(&txn, Column::IsAdmin, ids, false).await?
        + if to == Rang::Admin { given } else { 0 };
    if admins > 0 && dirigeants == 0 {
        return Err(refused(RefusalReason::NoRight));
    }

    let (superuser, dirigeant, admin) = flags(to);
    let written = user::Entity::update_many()
        .col_expr(Column::IsSuperuser, Expr::value(superuser))
        .col_expr(Column::IsDirigeant, Expr::value(dirigeant))
        .col_expr(Column::IsAdmin, Expr::value(admin))
        .filter(Column::Id.is_in(ids.iter().copied()))
        .filter(Column::Id.ne(actor.id))
        .filter(reach)
        .exec(&txn)
        .await?;
    if written.rows_affected != given {
        return Err(refused(RefusalReason::StaleRow));
    }
    // Leaving the admin rank drops the account's own rights: a later
    // promotion starts from none.
    if !leaving_admin.is_empty() {
        groupes_droits::Entity::delete_many()
            .filter(groupes_droits::Column::UserId.is_in(leaving_admin.iter().copied()))
            .exec(&txn)
            .await?;
    }

    let batch = uuid::Uuid::new_v4().to_string();
    for target in &targets {
        let summary = serde_json::json!({
            "rang": { "old": format!("{:?}", Rang::from(target)), "new": format!("{to:?}") }
        });
        log_action(
            &txn,
            ActionLog {
                user_id: actor.id,
                username: &actor.username,
                resource_key: resource_key::USERS,
                object_pk: &target.id.to_string(),
                object_repr: &target.username,
                action: "rank",
                summary: Some(summary.to_string()),
                batch_id: (targets.len() > 1).then(|| batch.clone()),
            },
        )
        .await?;
    }
    txn.commit().await?;
    Ok(given)
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::auth::gate::test_support::fixtures::{as_rank, groups_db, pk};
    use crate::auth::history;
    use crate::auth::permissions::groupe;
    use sea_orm::{ActiveModelTrait, Set};

    async fn rank_of(db: &ADb, n: u8) -> Rang {
        Rang::from(
            &user::Entity::find_by_id(pk(n))
                .one(db)
                .await
                .unwrap()
                .unwrap(),
        )
    }

    async fn responsable(db: &ADb) -> Pk {
        groupe::Entity::find_by_id(1)
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .responsable_id
    }

    /// One row of the admin's (33) own grid.
    async fn grid_row(db: &ADb) {
        groupes_droits::ActiveModel {
            groupe_id: Set(None),
            user_id: Set(Some(pk(33))),
            resource_key: Set("notes".into()),
            colonne: Set(String::new()),
            can_create: Set(false),
            can_read: Set(true),
            can_read_own: Set(false),
            can_update: Set(false),
            can_update_own: Set(false),
            can_delete: Set(false),
            can_delete_own: Set(false),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();
    }

    async fn grid_rows(db: &ADb) -> u64 {
        groupes_droits::Entity::find()
            .filter(groupes_droits::Column::UserId.eq(pk(33)))
            .count(db)
            .await
            .unwrap()
    }

    /// Only from dirigeant up, never onto oneself nor nobody.
    #[tokio::test]
    async fn a_rank_is_given_from_dirigeant_up_to_others_only() {
        let db = groups_db(pk(30)).await;
        let admin = as_rank(33, Rang::Admin);
        assert!(
            set_rank(&admin, &[pk(31)], Rang::Lambda, &db)
                .await
                .is_err()
        );
        let dirigeant = as_rank(32, Rang::Dirigeant);
        assert!(set_rank(&dirigeant, &[], Rang::Lambda, &db).await.is_err());
        assert!(
            set_rank(&dirigeant, &[pk(32), pk(31)], Rang::Lambda, &db)
                .await
                .is_err()
        );
        assert_eq!(rank_of(&db, 31).await, Rang::Lambda);
        set_rank(&dirigeant, &[pk(31)], Rang::Admin, &db)
            .await
            .unwrap();
        assert_eq!(rank_of(&db, 31).await, Rang::Admin);
    }

    /// A rank kept is no loss: the groups stay, the admin's grid too. Losing
    /// the admin rank takes both: the groups back, the grid gone.
    #[tokio::test]
    async fn only_a_lower_rank_takes_the_groups_and_the_grid() {
        let db = groups_db(pk(33)).await;
        grid_row(&db).await;
        let dirigeant = as_rank(32, Rang::Dirigeant);
        set_rank(&dirigeant, &[pk(33)], Rang::Admin, &db)
            .await
            .unwrap();
        assert_eq!(responsable(&db).await, pk(33));
        assert_eq!(grid_rows(&db).await, 1);

        set_rank(&dirigeant, &[pk(33)], Rang::Lambda, &db)
            .await
            .unwrap();
        assert_eq!(responsable(&db).await, pk(32));
        assert_eq!(grid_rows(&db).await, 0);
    }

    /// Admins never stay without an active dirigeant; without any admin, a
    /// change needs none.
    #[tokio::test]
    async fn the_last_dirigeant_is_kept_only_while_admins_remain() {
        let db = groups_db(pk(30)).await;
        let su = as_rank(30, Rang::Superuser);
        // No admin, no dirigeant left: a lambda stays a lambda freely.
        set_rank(&su, &[pk(33), pk(32)], Rang::Lambda, &db)
            .await
            .unwrap();
        set_rank(&su, &[pk(31)], Rang::Lambda, &db).await.unwrap();
        // A first admin with no dirigeant above him: refused.
        assert!(set_rank(&su, &[pk(31)], Rang::Admin, &db).await.is_err());
        assert_eq!(rank_of(&db, 31).await, Rang::Lambda);
    }

    /// One account changed: one history row, no batch; several: one batch.
    #[tokio::test]
    async fn several_accounts_share_one_batch() {
        let db = groups_db(pk(30)).await;
        let dirigeant = as_rank(32, Rang::Dirigeant);
        set_rank(&dirigeant, &[pk(31)], Rang::Admin, &db)
            .await
            .unwrap();
        set_rank(&dirigeant, &[pk(31), pk(33)], Rang::Lambda, &db)
            .await
            .unwrap();
        let rows = history::Entity::find()
            .filter(history::Column::Action.eq("rank"))
            .all(&db)
            .await
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows[0].batch_id.is_none());
        assert!(rows[1].batch_id.is_some() && rows[1].batch_id == rows[2].batch_id);
    }
}
