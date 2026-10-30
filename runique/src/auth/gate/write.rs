//! The admin's writes. Each takes the proof of ITS operation and consumes it.
//!
//! An update or a delete replays the proof's conditions in its `WHERE` and
//! must touch exactly one row: if the row changed between the gate and the
//! write (new owner, promoted out of reach, deleted), nothing is written.
use super::PkOf;
use super::operation::{Create, Delete, Update};
use super::proof::Authorized;
use super::refusal::{Refusal, RefusalReason};
use sea_orm::sea_query::IntoValueTuple;
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ActiveValue, ColumnTrait, Condition, ConnectionTrait,
    EntityTrait, IdenStatic, IntoActiveModel, Iterable, PrimaryKeyToColumn, QueryFilter, Value,
};

fn write_failed(e: sea_orm::DbErr) -> Refusal {
    tracing::warn!(error = %e, "admin write refused by the database");
    Refusal::new(RefusalReason::WriteFailed)
}

/// A blocked foreign key, from the driver's error kind when the engine is
/// compiled in, otherwise from the English wording the three engines share.
/// The wording stays in the log; the caller shows the `forms.delete_linked`
/// sentence and never a table name.
pub(crate) fn is_foreign_key_violation(err: &sea_orm::DbErr) -> bool {
    if driver_reports_foreign_key(err) {
        return true;
    }
    err.to_string()
        .to_ascii_lowercase()
        .contains("foreign key constraint")
}

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
fn driver_reports_foreign_key(err: &sea_orm::DbErr) -> bool {
    let runtime = match err {
        sea_orm::DbErr::Exec(err) | sea_orm::DbErr::Query(err) | sea_orm::DbErr::Conn(err) => err,
        _ => return false,
    };
    let sea_orm::RuntimeErr::SqlxError(err) = runtime else {
        return false;
    };
    err.as_database_error()
        .is_some_and(|db| db.is_foreign_key_violation())
}

#[cfg(not(any(feature = "sqlite", feature = "postgres", feature = "mysql")))]
fn driver_reports_foreign_key(_: &sea_orm::DbErr) -> bool {
    false
}

/// A delete the engine refused because another row still points here.
/// Any other error is returned unchanged. The engine text is logged, never shown.
pub(crate) fn linked_delete_error(err: sea_orm::DbErr) -> sea_orm::DbErr {
    if is_foreign_key_violation(&err) {
        tracing::warn!(error = %err, "delete blocked by a foreign key");
        sea_orm::DbErr::Custom(crate::utils::trad::t("forms.delete_linked").into_owned())
    } else {
        err
    }
}

/// `primary key = id`, composite keys included.
pub(in crate::auth::gate) fn pk_condition<E: EntityTrait>(id: PkOf<E>) -> Condition {
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

/// A generic write must not carry `is_superuser`, `is_dirigeant` or `is_admin`.
/// Giving or taking a rank goes through `set_rank`: the ceiling, the last
/// active dirigeant, the groups and the history live there. An insert may
/// leave the columns unset, or set them `false` (the table default, a lambda).
/// An update may not set them at all: `false` on an admin is a demotion.
fn rank_flags_block<A: ActiveModelTrait>(model: &A, update: bool) -> bool {
    const FLAGS: [&str; 3] = ["is_superuser", "is_dirigeant", "is_admin"];
    <<A as ActiveModelTrait>::Entity as EntityTrait>::Column::iter().any(|column| {
        if !FLAGS.contains(&column.as_str()) {
            return false;
        }
        match model.get(column) {
            ActiveValue::Set(Value::Bool(Some(false))) => update,
            ActiveValue::Set(_) => true,
            _ => false,
        }
    })
}

pub async fn insert<E, A, C>(
    proof: Authorized<Create, E>,
    model: A,
    db: &C,
) -> Result<E::Model, Refusal>
where
    E: EntityTrait,
    A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send,
    E::Model: IntoActiveModel<A>,
    C: ConnectionTrait,
{
    if rank_flags_block(&model, false) {
        return Err(Refusal::new(RefusalReason::RankNotWritable));
    }
    drop(proof);
    model.insert(db).await.map_err(write_failed)
}

/// Writes only the columns `changes` sets (`Set`), on the proof's row. The
/// model's `before_save` runs first: `update_many` skips it, and it stamps
/// the `auto_now_update` columns.
pub async fn update<E, A, C>(
    proof: Authorized<Update, E>,
    changes: A,
    db: &C,
) -> Result<(), Refusal>
where
    E: EntityTrait,
    A: ActiveModelTrait<Entity = E> + ActiveModelBehavior + Send,
    C: ConnectionTrait,
{
    if rank_flags_block(&changes, true) {
        return Err(Refusal::new(RefusalReason::RankNotWritable));
    }
    let (Some(id), conditions) = proof.into_parts() else {
        return Err(Refusal::new(RefusalReason::StaleRow));
    };
    let changes = changes.before_save(db, false).await.map_err(write_failed)?;
    if rank_flags_block(&changes, true) {
        return Err(Refusal::new(RefusalReason::RankNotWritable));
    }
    let result = E::update_many()
        .set(changes)
        .filter(pk_condition::<E>(id))
        .filter(conditions)
        .exec(db)
        .await
        .map_err(write_failed)?;
    one_row(result.rows_affected)
}

pub async fn delete<E: EntityTrait, C: ConnectionTrait>(
    proof: Authorized<Delete, E>,
    db: &C,
) -> Result<(), Refusal> {
    let (Some(id), conditions) = proof.into_parts() else {
        return Err(Refusal::new(RefusalReason::StaleRow));
    };
    let result = E::delete_many()
        .filter(pk_condition::<E>(id))
        .filter(conditions)
        .exec(db)
        .await
        .map_err(|error| {
            if is_foreign_key_violation(&error) {
                tracing::warn!(error = %error, "delete blocked by a foreign key");
                Refusal::new(RefusalReason::LinkedRecords)
            } else {
                write_failed(error)
            }
        })?;
    one_row(result.rows_affected)
}

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::gate::check::{GateCtx, authorize_create, authorize_row};
    use crate::auth::gate::test_support::fixtures::{db, note, pk, staff};
    use crate::auth::user;
    use crate::utils::aliases::ADb;
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
        let owner = staff(
            pk(10),
            "notes",
            [false, false, false, false, false, true, false],
        );
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
        let owner = staff(
            pk(10),
            "notes",
            [false, false, false, false, false, true, false],
        );
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
        let owner = staff(
            pk(10),
            "notes",
            [false, false, false, false, false, false, true],
        );
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
        let editor = staff(
            pk(10),
            "notes",
            [false, false, false, false, true, false, false],
        );
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
        let mut manager = staff(
            pk(40),
            "users",
            [false, true, false, true, false, false, false],
        );

        manager.rang = crate::auth::permissions::rang::Rang::Admin;
        manager.droits = manager.groupes[0].permissions.clone();

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
        let creator = staff(
            pk(10),
            "notes",
            [true, false, false, false, false, false, false],
        );
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

    fn accounts_manager() -> crate::auth::current_user::CurrentUser {
        let mut manager = staff(
            pk(40),
            "users",
            [true, true, false, true, false, false, false],
        );
        manager.rang = crate::auth::permissions::rang::Rang::Superuser;
        manager.droits = manager.groupes[0].permissions.clone();
        manager
    }

    async fn account(db: &ADb, id: crate::utils::pk::Pk) -> user::Model {
        user::Entity::find_by_id(id).one(db).await.unwrap().unwrap()
    }

    /// A superuser's proof would reach this lambda. The flag still does not
    /// go out with the email: both stay as they were.
    #[tokio::test]
    async fn an_update_that_sets_a_rank_flag_writes_nothing() {
        let db = db().await;
        let manager = accounts_manager();
        let ctx = GateCtx {
            db: &db,
            user: &manager,
        };
        let proof = authorize_row::<Update, user::Entity>(&ctx, pk(31))
            .await
            .unwrap();
        let err = update(
            proof,
            user::ActiveModel {
                email: Set("taken@evil.example".into()),
                is_dirigeant: Set(true),
                ..Default::default()
            },
            &db,
        )
        .await
        .unwrap_err();
        assert_eq!(*err.reason(), RefusalReason::RankNotWritable);
        let row = account(&db, pk(31)).await;
        assert_eq!(row.email, "member@example.com");
        assert!(!row.is_dirigeant);
        assert!(!row.is_admin);
        assert!(!row.is_superuser);
    }

    /// `false` on an admin is a demotion, and it does not belong here either.
    #[tokio::test]
    async fn an_update_that_clears_a_rank_flag_writes_nothing() {
        let db = db().await;
        let manager = accounts_manager();
        let ctx = GateCtx {
            db: &db,
            user: &manager,
        };
        let proof = authorize_row::<Update, user::Entity>(&ctx, pk(33))
            .await
            .unwrap();
        let err = update(
            proof,
            user::ActiveModel {
                is_admin: Set(false),
                ..Default::default()
            },
            &db,
        )
        .await
        .unwrap_err();
        assert_eq!(*err.reason(), RefusalReason::RankNotWritable);
        assert!(account(&db, pk(33)).await.is_admin);
    }

    #[tokio::test]
    async fn an_insert_that_sets_a_rank_flag_creates_nothing() {
        let db = db().await;
        let manager = accounts_manager();
        let proof = authorize_create::<user::Entity>(&GateCtx {
            db: &db,
            user: &manager,
        })
        .unwrap();
        let err = insert(
            proof,
            user::ActiveModel {
                username: Set("neo".into()),
                email: Set("neo@example.com".into()),
                password: Set("x".into()),
                is_superuser: Set(true),
                ..Default::default()
            },
            &db,
        )
        .await
        .unwrap_err();
        assert_eq!(*err.reason(), RefusalReason::RankNotWritable);
        let found = user::Entity::find()
            .filter(user::Column::Username.eq("neo"))
            .one(&*db)
            .await
            .unwrap();
        assert!(found.is_none());
    }

    /// Explicit `false` is the table default: the new account is a lambda.
    #[tokio::test]
    async fn an_insert_that_only_clears_rank_flags_creates_a_lambda() {
        let db = db().await;
        let manager = accounts_manager();
        let proof = authorize_create::<user::Entity>(&GateCtx {
            db: &db,
            user: &manager,
        })
        .unwrap();
        let row = insert(
            proof,
            user::ActiveModel {
                username: Set("neo".into()),
                email: Set("neo@example.com".into()),
                password: Set("x".into()),
                is_active: Set(false),
                is_superuser: Set(false),
                is_dirigeant: Set(false),
                is_admin: Set(false),
                ..Default::default()
            },
            &db,
        )
        .await
        .unwrap();
        assert!(!row.is_superuser);
        assert!(!row.is_dirigeant);
        assert!(!row.is_admin);
    }
}

#[cfg(test)]
mod foreign_key_tests {
    use super::{is_foreign_key_violation, linked_delete_error};
    use sea_orm::DbErr;

    #[test]
    fn the_three_engines_are_recognised_and_a_unique_violation_is_not() {
        let blocked = [
            "FOREIGN KEY constraint failed",
            "update or delete on table \"posts\" violates foreign key constraint \"comments_post_id_posts_fkey\"",
            "Cannot delete or update a parent row: a foreign key constraint fails (`demo`.`comments`, CONSTRAINT `comments_post_id_posts_fkey` FOREIGN KEY (`post_id`) REFERENCES `posts` (`id`))",
        ];
        for sample in blocked {
            let err = DbErr::Custom(sample.to_string());
            assert!(is_foreign_key_violation(&err), "{sample}");
            let shown = linked_delete_error(err).to_string();
            assert!(
                !shown.to_ascii_lowercase().contains("foreign key"),
                "{shown}"
            );
            assert!(!shown.contains("posts"), "{shown}");
            assert!(!shown.contains("comments"), "{shown}");
        }
        let unique = DbErr::Custom(
            "duplicate key value violates unique constraint \"users_email_key\"".to_string(),
        );
        assert!(!is_foreign_key_violation(&unique));
        assert!(
            matches!(linked_delete_error(unique), DbErr::Custom(message) if message.contains("unique"))
        );
    }
}
