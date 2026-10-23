//! The async gate: reads what the decision needs from the database, decides
//! (`decide`), and hands out the proof with the conditions it checked.
use super::PkOf;
use super::decide::{Ownership, rank_allows, rights_allow};
use super::operation::{AdminOperation, Create, List, View};
use super::proof::{Authorized, ReadableIds};
use super::refusal::{Refusal, RefusalReason};
use super::rights::{Crud, CrudAction, ReadableColumns};
use super::write::pk_condition;
use crate::auth::current_user::CurrentUser;
use crate::auth::meta::ModelMeta;
use crate::utils::{aliases::ADb, pk::Pk};
use sea_orm::{
    ColumnTrait, Condition, IdenStatic, Iterable, PrimaryKeyToColumn, QueryFilter, QuerySelect,
};

/// Who asks, and where to look the rows up.
pub struct GateCtx<'a> {
    pub db: &'a ADb,
    pub user: &'a CurrentUser,
}

/// A lookup the gate needs failed: refused, never let through.
fn lookup_failed(e: sea_orm::DbErr) -> Refusal {
    tracing::warn!(error = %e, "admin gate: lookup failed, action refused");
    Refusal::new(RefusalReason::LookupFailed)
}

/// Creating a row of `E`: the create right, nothing to look up.
pub fn authorize_create<E: ModelMeta>(ctx: &GateCtx<'_>) -> Result<Authorized<Create, E>, Refusal> {
    let rights = CrudAction::resolve(ctx.user, E::KEY);
    rights_allow::<Create>(&rights, Ownership::NotApplicable)?;
    Ok(Authorized::new(None, Condition::all()))
}

/// The operation `O` on the row `id` of `E`.
///
/// The full right opens any row; an `_own` right opens the row only when its
/// owner column holds the user's id, read here. On the account table, an
/// operation that protects accounts only reaches a strictly lower rank: the
/// condition goes into the WHERE, the target is never read beforehand.
/// Whatever was checked goes into the proof's conditions, replayed by the write.
pub async fn authorize_row<O: AdminOperation, E: ModelMeta>(
    ctx: &GateCtx<'_>,
    id: PkOf<E>,
) -> Result<Authorized<O, E>, Refusal> {
    let db = &**ctx.db;
    let rights = CrudAction::resolve(ctx.user, E::KEY);
    let mut conditions = Condition::all();

    let ownership = match (O::RIGHT_OWN, E::OWNER) {
        (Some(own), Some(owner_col)) if !rights.has(O::RIGHT) && rights.has(own) => {
            let owner: Option<Option<Pk>> = E::find_by_id(id.clone())
                .select_only()
                .column(owner_col)
                .into_tuple()
                .one(db)
                .await
                .map_err(lookup_failed)?;
            if owner.flatten() == Some(ctx.user.id) {
                conditions = conditions.add(owner_col.eq(ctx.user.id));
                Ownership::Owner
            } else {
                Ownership::NotOwner
            }
        }
        _ => Ownership::NotApplicable,
    };
    rights_allow::<O>(&rights, ownership)?;

    if E::IS_ACCOUNT_TABLE
        && let Some(cibles) = rank_allows::<O>(ctx.user.rang)?
    {
        conditions = conditions.add(cibles);
    }

    Ok(Authorized::new(Some(id), conditions))
}

/// The rows of `E` the user may read: all of them with the full right, their
/// own with the `_own` right (if `E` has an owner column), none otherwise.
fn read_scope<E: ModelMeta>(ctx: &GateCtx<'_>) -> Result<Condition, Refusal> {
    let rights = CrudAction::resolve(ctx.user, E::KEY);
    if rights.has(Crud::Read) {
        Ok(Condition::all())
    } else if rights.has(Crud::ReadOwn)
        && let Some(owner_col) = E::OWNER
    {
        Ok(Condition::all().add(owner_col.eq(ctx.user.id)))
    } else {
        Err(Refusal::new(RefusalReason::NoRight))
    }
}

/// The columns of `E` the user may read, refused when none is but the key: a page of bare keys shows nothing,
/// and still says wich rows exists. A secret column never counts.
fn readable_columns<E: ModelMeta>(ctx: &GateCtx<'_>) -> Result<ReadableColumns, Refusal> {
    let cols = ReadableColumns::resolve(ctx.user, E::KEY);
    let keys: Vec<&str> = E::PrimaryKey::iter()
        .map(|k| k.into_column().as_str())
        .collect();
    let one = E::Column::iter().any(|c| {
        let name = c.as_str();
        !keys.contains(&name) && !E::SECRET.iter().any(|s| s.as_str() == name) && cols.allows(name)
    });
    if one {
        Ok(cols)
    } else {
        Err(Refusal::new(RefusalReason::NoReadableColumn))
    }
}
/// Listing `E`. The proof carries the read scope: the list query and its
/// count both apply it, so a page never shows a row the user can't read.
pub fn authorize_list<E: ModelMeta>(ctx: &GateCtx<'_>) -> Result<Authorized<List, E>, Refusal> {
    let scope = read_scope::<E>(ctx)?;
    Ok(Authorized::new(None, scope).reading(readable_columns::<E>(ctx)?))
}

/// Viewing the row `id` of `E`. Nothing is looked up beforehand: the proof
/// carries the read scope, and the read filtered by it is the check. A row
/// out of scope comes back as no row, refused like a missing one.
pub fn authorize_view<E: ModelMeta>(
    ctx: &GateCtx<'_>,
    id: PkOf<E>,
) -> Result<Authorized<View, E>, Refusal> {
    let scope = read_scope::<E>(ctx)?;
    Ok(Authorized::new(Some(id), scope).reading(readable_columns::<E>(ctx)?))
}

/// Ids sent by the client for a foreign key or a many-to-many field, kept
/// only if the user can read every one of them: they exist and, for an
/// `_own` read right, belong to the user. One query for the whole field; one
/// id out of reach refuses the field.
pub async fn readable_ids<T: ModelMeta>(
    ctx: &GateCtx<'_>,
    ids: Vec<PkOf<T>>,
) -> Result<ReadableIds<T>, Refusal> {
    let mut unique: Vec<PkOf<T>> = Vec::with_capacity(ids.len());
    for id in ids {
        if !unique.contains(&id) {
            unique.push(id);
        }
    }
    if unique.is_empty() {
        return Ok(ReadableIds::new(unique));
    }

    let scope = read_scope::<T>(ctx)?;

    let wanted = unique
        .iter()
        .cloned()
        .fold(Condition::any(), |cond, id| cond.add(pk_condition::<T>(id)));
    let found = T::find()
        .filter(wanted)
        .filter(scope)
        .select_only()
        .columns(T::PrimaryKey::iter().map(|key| key.into_column()))
        .into_tuple::<PkOf<T>>()
        .all(&**ctx.db)
        .await
        .map_err(lookup_failed)?
        .len();
    if found != unique.len() {
        return Err(Refusal::new(RefusalReason::UnreadableTarget));
    }
    Ok(ReadableIds::new(unique))
}

#[cfg(feature = "sqlite")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::gate::operation::{Delete, ResetPassword, Update, View};
    use crate::auth::gate::proof::Guarded;
    use crate::auth::gate::test_support::fixtures::{db, note, pk, staff};
    use crate::utils::aliases::ADb;
    use sea_orm::{EntityTrait, PaginatorTrait};

    async fn visible_notes(ctx: &GateCtx<'_>) -> u64 {
        let proof = authorize_list::<note::Entity>(ctx).unwrap();
        note::Entity::find()
            .filter(proof.conditions())
            .count(&**ctx.db)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn the_read_right_lists_every_row() {
        let db = db().await;
        let reader = staff(
            pk(10),
            "notes",
            [false, true, false, false, false, false, false],
        );
        let ctx = GateCtx {
            db: &db,
            user: &reader,
        };
        assert_eq!(visible_notes(&ctx).await, 3);
    }

    #[tokio::test]
    async fn without_the_read_right_the_list_is_refused() {
        let db = db().await;
        let writer = staff(
            pk(10),
            "notes",
            [true, false, false, true, true, true, true],
        );
        let ctx = GateCtx {
            db: &db,
            user: &writer,
        };
        assert!(authorize_list::<note::Entity>(&ctx).is_err());
    }

    /// The full right opens the row as is: holding the `_own` right too must
    /// not narrow the write to the user's rows.
    #[tokio::test]
    async fn the_full_right_adds_no_owner_condition_even_with_the_own_right() {
        use sea_orm::{DbBackend, QueryTrait};
        let db = db().await;
        let both = staff(
            pk(10),
            "notes",
            [false, false, false, true, false, true, false],
        );
        let ctx = GateCtx {
            db: &db,
            user: &both,
        };
        let proof = authorize_row::<Update, note::Entity>(&ctx, 1)
            .await
            .unwrap();
        let sql = note::Entity::find()
            .filter(proof.conditions())
            .build(DbBackend::Sqlite)
            .to_string();
        let condition = sql.split_once("WHERE").map_or("", |(_, w)| w);
        assert!(!condition.contains("owner_id"), "{sql}");
    }

    #[tokio::test]
    async fn viewing_asks_the_read_right_and_looks_nothing_up() {
        let db = db().await;
        let reader = staff(
            pk(10),
            "notes",
            [false, true, false, false, false, false, false],
        );
        let writer = staff(
            pk(10),
            "notes",
            [true, false, false, true, true, true, true],
        );
        let proof = authorize_view::<note::Entity>(
            &GateCtx {
                db: &db,
                user: &reader,
            },
            99,
        )
        .unwrap();
        assert_eq!(
            proof.id(),
            Some(&99),
            "no lookup: a missing row still gets a proof"
        );
        assert!(
            authorize_view::<note::Entity>(
                &GateCtx {
                    db: &db,
                    user: &writer,
                },
                1,
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn a_right_on_another_resource_opens_nothing() {
        let db = db().await;
        let other = staff(pk(10), "users", [true, true, false, true, true, true, true]);
        let ctx = GateCtx {
            db: &db,
            user: &other,
        };
        assert!(authorize_list::<note::Entity>(&ctx).is_err());
    }

    #[tokio::test]
    async fn an_own_right_opens_the_users_row_and_no_other() {
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
        assert!(authorize_row::<Update, note::Entity>(&ctx, 1).await.is_ok());
        assert!(
            authorize_row::<Update, note::Entity>(&ctx, 2)
                .await
                .is_err(),
            "another's row"
        );
        assert!(
            authorize_row::<Update, note::Entity>(&ctx, 3)
                .await
                .is_err(),
            "no owner"
        );
        assert!(
            authorize_row::<Update, note::Entity>(&ctx, 99)
                .await
                .is_err(),
            "no row"
        );
        assert!(
            authorize_row::<Delete, note::Entity>(&ctx, 1)
                .await
                .is_err(),
            "update_own only"
        );
    }

    #[tokio::test]
    async fn the_full_right_opens_any_row_without_an_owner_condition() {
        let db = db().await;
        let editor = staff(
            pk(10),
            "notes",
            [false, false, false, true, false, false, false],
        );
        let ctx = GateCtx {
            db: &db,
            user: &editor,
        };
        for id in [1, 2, 3] {
            let proof = authorize_row::<Update, note::Entity>(&ctx, id)
                .await
                .unwrap();
            assert_eq!(proof.id(), Some(&id));
        }
    }

    /// The account in `rang` with every right on `users`: from its group and
    /// on its account, so each rank finds it where it reads it.
    fn manager(rang: crate::auth::permissions::rang::Rang) -> CurrentUser {
        let mut user = staff(
            pk(40),
            "users",
            [true, true, false, true, true, false, false],
        );
        user.droits = user.groupes[0].permissions.clone();
        user.rang = rang;
        user
    }

    async fn writes_email(
        db: &ADb,
        user: &CurrentUser,
        id: Pk,
    ) -> Result<(), crate::auth::gate::refusal::Refusal> {
        use crate::auth::gate::write::update;
        use sea_orm::Set;
        let ctx = GateCtx { db, user };
        let proof = authorize_row::<Update, crate::auth::user::Entity>(&ctx, id).await?;
        let changes = crate::auth::user::ActiveModel {
            email: Set("changed@example.com".into()),
            ..Default::default()
        };
        update(proof, changes, db).await
    }

    #[tokio::test]
    async fn an_account_without_rank_writes_no_account_but_reads_them() {
        use crate::auth::permissions::rang::Rang;
        let db = db().await;
        let staff = manager(Rang::Lambda);
        let ctx = GateCtx {
            db: &db,
            user: &staff,
        };
        let member = pk(31);
        assert!(
            authorize_row::<Update, crate::auth::user::Entity>(&ctx, member)
                .await
                .is_err()
        );
        assert!(
            authorize_row::<Delete, crate::auth::user::Entity>(&ctx, member)
                .await
                .is_err()
        );
        assert!(
            authorize_row::<ResetPassword, crate::auth::user::Entity>(&ctx, member)
                .await
                .is_err()
        );
        assert!(
            authorize_row::<View, crate::auth::user::Entity>(&ctx, pk(30))
                .await
                .is_err(),
            "a group never opens a framework table, reading included"
        );
    }

    /// Accounts: the superuser `pk(30)`, the lambda `pk(31)`, the dirigeant
    /// `pk(32)`, the admin `pk(33)`.
    #[tokio::test]
    async fn each_rank_writes_only_strictly_lower_accounts() {
        use crate::auth::permissions::rang::Rang;
        let cases = [
            (Rang::Admin, [false, true, false, false]),
            (Rang::Dirigeant, [false, true, false, true]),
            (Rang::Superuser, [false, true, true, true]),
        ];
        for (rang, expected) in cases {
            let user = manager(rang);
            for (target, allowed) in [pk(30), pk(31), pk(32), pk(33)].into_iter().zip(expected) {
                let db = db().await;
                assert_eq!(
                    writes_email(&db, &user, target).await.is_ok(),
                    allowed,
                    "{rang:?} on {target}"
                );
            }
        }
    }

    /// Never oneself: the account's own row is of its own rank.
    #[tokio::test]
    async fn a_rank_never_writes_its_own_account() {
        use crate::auth::permissions::rang::Rang;
        let db = db().await;
        let mut admin = manager(Rang::Admin);
        admin.id = pk(33);
        assert!(writes_email(&db, &admin, pk(33)).await.is_err());
    }

    #[tokio::test]
    async fn create_asks_the_create_right_only() {
        let db = db().await;
        let creator = staff(
            pk(10),
            "notes",
            [true, false, false, false, false, false, false],
        );
        let reader = staff(
            pk(10),
            "notes",
            [false, true, false, true, true, true, true],
        );
        assert!(
            authorize_create::<note::Entity>(&GateCtx {
                db: &db,
                user: &creator
            })
            .is_ok()
        );
        assert!(
            authorize_create::<note::Entity>(&GateCtx {
                db: &db,
                user: &reader
            })
            .is_err()
        );
    }

    #[tokio::test]
    async fn readable_ids_keeps_a_field_only_if_every_target_is_readable() {
        let db = db().await;
        let reader = staff(
            pk(10),
            "notes",
            [false, true, false, false, false, false, false],
        );
        let ctx = GateCtx {
            db: &db,
            user: &reader,
        };
        let kept = readable_ids::<note::Entity>(&ctx, vec![1, 2, 2])
            .await
            .unwrap();
        assert_eq!(kept.ids(), &[1, 2], "deduplicated");
        assert!(
            readable_ids::<note::Entity>(&ctx, vec![1, 99])
                .await
                .is_err(),
            "one missing target refuses the field"
        );

        let blind = staff(
            pk(10),
            "notes",
            [false, false, false, true, true, true, true],
        );
        let ctx = GateCtx {
            db: &db,
            user: &blind,
        };
        assert!(
            readable_ids::<note::Entity>(&ctx, vec![1]).await.is_err(),
            "no read right"
        );
        assert!(
            readable_ids::<note::Entity>(&ctx, vec![]).await.is_ok(),
            "an empty field reads nothing"
        );
    }
    /// Replaces the columns of the user's only group right.
    fn columns(user: &mut CurrentUser, cols: &[(&str, bool)]) {
        use crate::auth::permissions::ColumnRight;
        user.groupes[0].permissions[0].colonnes = cols
            .iter()
            .map(|&(name, read)| ColumnRight {
                colonne: name.into(),
                can_read: read,
                can_create: false,
                can_update: true,
            })
            .collect();
    }

    #[tokio::test]
    async fn a_read_needs_one_readable_column_beyond_the_key() {
        let db = db().await;
        let mut reader = staff(
            pk(10),
            "notes",
            [false, true, false, false, false, false, false],
        );
        let cases: [(&[(&str, bool)], bool); 4] = [
            (&[], false),
            (&[("owner_id", false)], false),
            (&[("id", true)], false),
            (&[("owner_id", true)], true),
        ];
        for (cols, readable) in cases {
            columns(&mut reader, cols);
            let ctx = GateCtx {
                db: &db,
                user: &reader,
            };
            match authorize_list::<note::Entity>(&ctx) {
                Ok(proof) => {
                    assert!(readable, "{cols:?}");
                    assert!(proof.readable().allows("owner_id"));
                }
                Err(refusal) => {
                    assert!(!readable, "{cols:?}");
                    assert_eq!(*refusal.reason(), RefusalReason::NoReadableColumn);
                }
            }
            assert_eq!(
                authorize_view::<note::Entity>(&ctx, 1).is_ok(),
                readable,
                "{cols:?}"
            );
        }
    }

    #[tokio::test]
    async fn the_superuser_and_the_dirigeant_read_every_column() {
        use crate::auth::permissions::rang::Rang;
        let db = db().await;
        for rang in [Rang::Superuser, Rang::Dirigeant] {
            let mut user = staff(pk(10), "nothing", [false; 7]);
            user.rang = rang;
            let ctx = GateCtx {
                db: &db,
                user: &user,
            };
            let proof = authorize_list::<note::Entity>(&ctx).unwrap();
            assert_eq!(proof.readable(), &ReadableColumns::All, "{rang:?}");
        }
    }

    #[tokio::test]
    async fn a_secret_column_never_counts_as_readable() {
        use crate::auth::permissions::rang::Rang;
        let db = db().await;
        let mut admin = staff(
            pk(40),
            "users",
            [false, true, false, false, false, false, false],
        );
        admin.rang = Rang::Admin;
        columns(&mut admin, &[("password", true)]);
        admin.droits = admin.groupes[0].permissions.clone();
        let ctx = GateCtx {
            db: &db,
            user: &admin,
        };
        assert!(authorize_list::<crate::auth::user::Entity>(&ctx).is_err());

        columns(&mut admin, &[("password", true), ("username", true)]);
        admin.droits = admin.groupes[0].permissions.clone();
        let ctx = GateCtx {
            db: &db,
            user: &admin,
        };
        assert!(authorize_list::<crate::auth::user::Entity>(&ctx).is_ok());
    }
}
