//! The admin's writes. Each takes the proof of ITS operation and consumes it.
//!
//! An update or a delete replays the proof's conditions in its `WHERE` and
//! must touch exactly one row: if the row changed between the gate and the
//! write (new owner, promoted out of reach, deleted), nothing is written.
use super::PkOf;
use super::operation::{Create, Delete, Update};
use super::proof::Authorized;
use super::refusal::{Refusal, RefusalReason};
use crate::utils::aliases::ADb;
use sea_orm::sea_query::IntoValueTuple;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, Condition, EntityTrait, IntoActiveModel,
    Iterable, PrimaryKeyToColumn, QueryFilter,
};

fn write_failed(e: sea_orm::DbErr) -> Refusal {
    tracing::warn!(error = %e, "admin write refused by the database");
    Refusal::new(RefusalReason::WriteFailed)
}

/// `primary key = id`, composite keys included.
pub(in crate::admin::refonte_admin::gate) fn pk_condition<E: EntityTrait>(
    id: PkOf<E>,
) -> Condition {
    E::PrimaryKey::iter()
        .zip(id.into_value_tuple())
        .fold(Condition::all(), |cond, (key, value)| {
            cond.add(key.into_column().eq(value))
        })
}

fn one_row(rows_affected: u64) -> Result<(), Refusal> {
    if rows_affected == 1 {
        Ok(())
    } else {
        Err(Refusal::new(RefusalReason::StaleRow))
    }
}

pub async fn insert<E, A>(
    proof: Authorized<Create, E>,
    model: A,
    db: &ADb,
) -> Result<E::Model, Refusal>
where
    E: EntityTrait,
    A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send,
    E::Model: IntoActiveModel<A>,
{
    drop(proof);
    model.insert(&**db).await.map_err(write_failed)
}

/// Writes only the columns `changes` sets (`Set`), on the proof's row.
pub async fn update<E, A>(proof: Authorized<Update, E>, changes: A, db: &ADb) -> Result<(), Refusal>
where
    E: EntityTrait,
    A: ActiveModelTrait<Entity = E> + Send,
{
    let (Some(id), conditions) = proof.into_parts() else {
        return Err(Refusal::new(RefusalReason::StaleRow));
    };
    let result = E::update_many()
        .set(changes)
        .filter(pk_condition::<E>(id))
        .filter(conditions)
        .exec(&**db)
        .await
        .map_err(write_failed)?;
    one_row(result.rows_affected)
}

pub async fn delete<E: EntityTrait>(proof: Authorized<Delete, E>, db: &ADb) -> Result<(), Refusal> {
    let (Some(id), conditions) = proof.into_parts() else {
        return Err(Refusal::new(RefusalReason::StaleRow));
    };
    let result = E::delete_many()
        .filter(pk_condition::<E>(id))
        .filter(conditions)
        .exec(&**db)
        .await
        .map_err(write_failed)?;
    one_row(result.rows_affected)
}

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::refonte_admin::gate::check::{GateCtx, authorize_create, authorize_row};
    use crate::admin::refonte_admin::gate::test_support::fixtures::{db, note, pk, staff};
    use crate::auth::user;
    use sea_orm::Set;

    async fn owner_of(db: &ADb, id: i32) -> Option<Option<crate::utils::pk::Pk>> {
        note::Entity::find_by_id(id)
            .one(&**db)
            .await
            .unwrap()
            .map(|n| n.owner_id)
    }

    async fn reassign(db: &ADb, id: i32, owner: crate::utils::pk::Pk) {
        note::ActiveModel {
            id: Set(id),
            owner_id: Set(Some(owner)),
        }
        .update(&**db)
        .await
        .unwrap();
    }

    fn new_owner(owner: crate::utils::pk::Pk) -> note::ActiveModel {
        note::ActiveModel {
            owner_id: Set(Some(owner)),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn an_own_update_lands_while_the_row_is_still_the_users() {
        let db = db().await;
        let owner = staff(pk(10), "notes", [false, false, false, false, true, false]);
        let ctx = GateCtx {
            db: &db,
            user: &owner,
        };
        let proof = authorize_row::<Update, note::Entity>(&ctx, 1)
            .await
            .unwrap();
        update(proof, new_owner(pk(10)), &db).await.unwrap();
        assert_eq!(owner_of(&db, 1).await, Some(Some(pk(10))));
    }

    #[tokio::test]
    async fn an_own_update_is_refused_if_the_row_changed_owner_in_between() {
        let db = db().await;
        let owner = staff(pk(10), "notes", [false, false, false, false, true, false]);
        let ctx = GateCtx {
            db: &db,
            user: &owner,
        };
        let proof = authorize_row::<Update, note::Entity>(&ctx, 1)
            .await
            .unwrap();
        reassign(&db, 1, pk(20)).await;
        assert!(update(proof, new_owner(pk(10)), &db).await.is_err());
        assert_eq!(
            owner_of(&db, 1).await,
            Some(Some(pk(20))),
            "nothing written"
        );
    }

    #[tokio::test]
    async fn an_own_delete_is_refused_if_the_row_changed_owner_in_between() {
        let db = db().await;
        let owner = staff(pk(10), "notes", [false, false, false, false, false, true]);
        let ctx = GateCtx {
            db: &db,
            user: &owner,
        };
        let proof = authorize_row::<Delete, note::Entity>(&ctx, 1)
            .await
            .unwrap();
        reassign(&db, 1, pk(20)).await;
        assert!(delete(proof, &db).await.is_err());
        assert!(owner_of(&db, 1).await.is_some(), "row kept");

        let proof = authorize_row::<Delete, note::Entity>(&ctx, 1).await;
        assert!(proof.is_err(), "no longer the user's");
    }

    #[tokio::test]
    async fn a_delete_lands_once_and_a_vanished_row_is_refused() {
        let db = db().await;
        let editor = staff(pk(10), "notes", [false, false, false, true, false, false]);
        let ctx = GateCtx {
            db: &db,
            user: &editor,
        };
        let first = authorize_row::<Delete, note::Entity>(&ctx, 2)
            .await
            .unwrap();
        let second = authorize_row::<Delete, note::Entity>(&ctx, 2)
            .await
            .unwrap();
        delete(first, &db).await.unwrap();
        assert!(owner_of(&db, 2).await.is_none());
        assert!(delete(second, &db).await.is_err(), "already gone");
    }

    #[tokio::test]
    async fn an_admin_update_is_refused_if_the_account_was_promoted_in_between() {
        let db = db().await;
        let mut manager = staff(pk(40), "users", [false, true, true, false, false, false]);
        manager.rang = crate::auth::rang::Rang::Admin;
        let ctx = GateCtx {
            db: &db,
            user: &manager,
        };
        let proof = authorize_row::<Update, user::Entity>(&ctx, pk(31))
            .await
            .unwrap();
        user::ActiveModel {
            id: Set(pk(31)),
            is_superuser: Set(true),
            ..Default::default()
        }
        .update(&*db)
        .await
        .unwrap();
        let changes = user::ActiveModel {
            email: Set("taken@evil.example".into()),
            ..Default::default()
        };
        assert!(update(proof, changes, &db).await.is_err());
        let account = user::Entity::find_by_id(pk(31))
            .one(&*db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(account.email, "member@example.com", "nothing written");
    }

    #[tokio::test]
    async fn an_insert_needs_and_consumes_a_create_proof() {
        let db = db().await;
        let creator = staff(pk(10), "notes", [true, false, false, false, false, false]);
        let proof = authorize_create::<note::Entity>(&GateCtx {
            db: &db,
            user: &creator,
        })
        .unwrap();
        let row = insert(
            proof,
            note::ActiveModel {
                id: Set(7),
                owner_id: Set(Some(pk(10))),
            },
            &db,
        )
        .await
        .unwrap();
        assert_eq!(row.id, 7);
        assert_eq!(owner_of(&db, 7).await, Some(Some(pk(10))));
    }
}
