//! A group handed to another responsable, decided by a dirigeant alone. The
//! receiver is an active account of rank admin or above, checked in the
//! write's own `WHERE`. Handed to an admin, the group keeps only what his
//! grid covers, stored: the rights its members lose are shown first, and the
//! write goes through only for the losses the dirigeant saw.
use super::refusal::{Refusal, RefusalReason};
use super::rights_grid::{describe, from_rows};
use crate::auth::current_user::CurrentUser;
use crate::auth::history::{ActionLog, log_action};
use crate::auth::permissions::rang::Rang;
use crate::auth::permissions::{group_rows_within, groupe, groupes_droits, users_groupes};
use crate::auth::user;
use crate::utils::aliases::ADb;
use crate::utils::constante::resource_key;
use crate::utils::pk::Pk;
use crate::utils::trad::t;
use sea_orm::sea_query::{Expr, Query};
use sea_orm::{
    ActiveValue, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect, TransactionTrait,
};

fn refused(reason: RefusalReason) -> DbErr {
    DbErr::Custom(Refusal::new(reason).message())
}

/// The refusal of a receiver that doesn't qualify. One message whatever the
/// cause: the selector offers only valid receivers, so this answers a forged
/// form or a race, and the cause goes to the logs.
fn unsuitable(cause: &str) -> DbErr {
    tracing::info!(cause, "group transfer refused: unsuitable receiver");
    DbErr::Custom(t("admin.transfer.unsuitable").into_owned())
}

/// The accounts that may receive a group: active, of rank admin or above.
fn may_receive() -> Condition {
    Condition::all().add(user::Column::IsActive.eq(true)).add(
        Condition::any()
            .add(user::Column::IsAdmin.eq(true))
            .add(user::Column::IsDirigeant.eq(true))
            .add(user::Column::IsSuperuser.eq(true)),
    )
}

/// The accounts group `group` may be handed to, by name.
pub(crate) async fn receivers<C: ConnectionTrait>(
    group: &groupe::Model,
    db: &C,
) -> Result<Vec<(Pk, String)>, DbErr> {
    user::Entity::find()
        .select_only()
        .columns([user::Column::Id, user::Column::Username])
        .filter(may_receive())
        .filter(user::Column::Id.ne(group.responsable_id))
        .order_by_asc(user::Column::Username)
        .into_tuple()
        .all(db)
        .await
}

/// Who took back each of the groups `ids` waiting for a responsable, by
/// group id. Two queries at most: the groups, then their takers' names.
pub(crate) async fn taken_by<C: ConnectionTrait>(
    ids: &[i32],
    db: &C,
) -> Result<std::collections::HashMap<i32, String>, DbErr> {
    let taken: Vec<(i32, Option<Pk>)> = groupe::Entity::find()
        .select_only()
        .columns([groupe::Column::Id, groupe::Column::ReprisPar])
        .filter(groupe::Column::Id.is_in(ids.iter().copied()))
        .filter(groupe::Column::ReprisPar.is_not_null())
        .into_tuple()
        .all(db)
        .await?;
    if taken.is_empty() {
        return Ok(Default::default());
    }
    let names: std::collections::HashMap<Pk, String> = user::Entity::find()
        .select_only()
        .columns([user::Column::Id, user::Column::Username])
        .filter(user::Column::Id.is_in(taken.iter().filter_map(|(_, by)| *by)))
        .into_tuple::<(Pk, String)>()
        .all(db)
        .await?
        .into_iter()
        .collect();
    Ok(taken
        .into_iter()
        .filter_map(|(id, by)| Some((id, names.get(&by?)?.clone())))
        .collect())
}

/// One right the group holds and the receiver doesn't: kept, the receiver
/// gets it and the members keep it; not kept, the members lose it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Line {
    pub resource_key: String,
    /// Empty for the table's right.
    pub colonne: String,
    pub right: &'static str,
    /// False for a full right on the groups: never given to an admin.
    pub keepable: bool,
}

/// What handing a group over does, before anything is written.
pub(crate) struct Plan {
    pub group: groupe::Model,
    pub from: String,
    pub to: user::Model,
    /// The rights to keep or lose, in the grid's order; empty when the
    /// receiver covers the whole group.
    pub lines: Vec<Line>,
    /// The same lines as the gate compares them: the page sends it back.
    pub losses: String,
    rows: Vec<groupes_droits::Model>,
    /// The receiver's own grid, when he is an admin.
    grid: Option<Vec<groupes_droits::Model>>,
}

/// The rows of `kept` as stored rows, for the history's description.
fn as_rows(kept: &[groupes_droits::ActiveModel]) -> Vec<groupes_droits::Model> {
    let flag = |v: &ActiveValue<bool>| v.clone().take().unwrap_or(false);
    kept.iter()
        .map(|a| groupes_droits::Model {
            id: 0,
            groupe_id: a.groupe_id.clone().take().flatten(),
            user_id: a.user_id.clone().take().flatten(),
            resource_key: a.resource_key.clone().take().unwrap_or_default(),
            colonne: a.colonne.clone().take().unwrap_or_default(),
            can_create: flag(&a.can_create),
            can_read: flag(&a.can_read),
            can_read_own: flag(&a.can_read_own),
            can_update: flag(&a.can_update),
            can_update_own: flag(&a.can_update_own),
            can_delete: flag(&a.can_delete),
            can_delete_own: flag(&a.can_delete_own),
        })
        .collect()
}

/// `row` as a new row to insert.
fn fresh(row: &groupes_droits::Model) -> groupes_droits::ActiveModel {
    use sea_orm::Set;
    groupes_droits::ActiveModel {
        groupe_id: Set(row.groupe_id),
        user_id: Set(row.user_id),
        resource_key: Set(row.resource_key.clone()),
        colonne: Set(row.colonne.clone()),
        can_create: Set(row.can_create),
        can_read: Set(row.can_read),
        can_read_own: Set(row.can_read_own),
        can_update: Set(row.can_update),
        can_update_own: Set(row.can_update_own),
        can_delete: Set(row.can_delete),
        can_delete_own: Set(row.can_delete_own),
        ..Default::default()
    }
}

/// The flag of `row` a right names.
fn flag_mut<'a>(row: &'a mut groupes_droits::Model, right: &str) -> Option<&'a mut bool> {
    Some(match right {
        "create" => &mut row.can_create,
        "read" => &mut row.can_read,
        "read_own" => &mut row.can_read_own,
        "update" => &mut row.can_update,
        "update_own" => &mut row.can_update_own,
        "delete" => &mut row.can_delete,
        "delete_own" => &mut row.can_delete_own,
        _ => return None,
    })
}

/// Each right `rows` grant that `kept` no longer does, in the grid's order.
fn lines(rows: &[groupes_droits::Model], kept: &[groupes_droits::Model]) -> Vec<Line> {
    const TABLE: [&str; 7] = [
        "create",
        "read",
        "read_own",
        "update",
        "update_own",
        "delete",
        "delete_own",
    ];
    const COLUMN: [&str; 3] = ["read", "create", "update"];
    let mut sorted: Vec<&groupes_droits::Model> = rows.iter().collect();
    sorted.sort_by(|a, b| (&a.resource_key, &a.colonne).cmp(&(&b.resource_key, &b.colonne)));
    let mut out = Vec::new();
    for row in sorted {
        let after = kept
            .iter()
            .find(|k| k.resource_key == row.resource_key && k.colonne == row.colonne);
        let rights: &[&'static str] = if row.colonne.is_empty() {
            &TABLE
        } else {
            &COLUMN
        };
        for right in rights {
            let had = flag_mut(&mut row.clone(), right).is_some_and(|f| *f);
            let keeps = after.is_some_and(|k| flag_mut(&mut k.clone(), right).is_some_and(|f| *f));
            if had && !keeps {
                let full_on_groups = row.resource_key == resource_key::GROUPES
                    && row.colonne.is_empty()
                    && matches!(*right, "read" | "update" | "delete");
                out.push(Line {
                    resource_key: row.resource_key.clone(),
                    colonne: row.colonne.clone(),
                    right,
                    keepable: !full_on_groups,
                });
            }
        }
    }
    out
}

/// `lines` as the gate compares them.
fn shown_of(lines: &[Line]) -> String {
    lines
        .iter()
        .map(|l| format!("{}.{}.{}", l.resource_key, l.colonne, l.right))
        .collect::<Vec<_>>()
        .join(",")
}

/// `grid`, the receiver's, with the rights of `keep` added.
fn widened(grid: &[groupes_droits::Model], to: Pk, keep: &[Line]) -> Vec<groupes_droits::Model> {
    let mut grid = grid.to_vec();
    for line in keep {
        let at = match grid
            .iter()
            .position(|r| r.resource_key == line.resource_key && r.colonne == line.colonne)
        {
            Some(at) => at,
            None => {
                grid.push(groupes_droits::Model {
                    id: 0,
                    groupe_id: None,
                    user_id: Some(to),
                    resource_key: line.resource_key.clone(),
                    colonne: line.colonne.clone(),
                    can_create: false,
                    can_read: false,
                    can_read_own: false,
                    can_update: false,
                    can_update_own: false,
                    can_delete: false,
                    can_delete_own: false,
                });
                grid.len() - 1
            }
        };
        if let Some(flag) = flag_mut(&mut grid[at], line.right) {
            *flag = true;
        }
    }
    grid
}

/// The transfer of group `groupe_id` to `to`, as `actor` would make it.
/// Refused for anyone below dirigeant, for a missing group, for a receiver
/// that doesn't qualify or already answers for it, and for an admin
/// receiving a group with ranked members (an admin's group holds site
/// accounts only).
pub(crate) async fn plan<C: ConnectionTrait>(
    actor: &CurrentUser,
    groupe_id: i32,
    to: Pk,
    db: &C,
) -> Result<Plan, DbErr> {
    if actor.rang < Rang::Dirigeant {
        return Err(refused(RefusalReason::RankTooLow));
    }
    let Some(group) = groupe::Entity::find_by_id(groupe_id).one(db).await? else {
        return Err(refused(RefusalReason::NotFound));
    };
    if to == group.responsable_id {
        return Err(unsuitable("already the responsable"));
    }
    let Some(receiver) = user::Entity::find_by_id(to)
        .filter(may_receive())
        .one(db)
        .await?
    else {
        return Err(unsuitable("missing, inactive or below admin"));
    };
    let from = user::Entity::find_by_id(group.responsable_id)
        .one(db)
        .await?
        .map(|u| u.username)
        .unwrap_or_default();
    let rows = groupes_droits::Entity::find()
        .filter(groupes_droits::Column::GroupeId.eq(groupe_id))
        .all(db)
        .await?;
    if Rang::from(&receiver) != Rang::Admin {
        let losses = String::new();
        return Ok(Plan {
            group,
            from,
            to: receiver,
            lines: Vec::new(),
            losses,
            rows,
            grid: None,
        });
    }

    let members: Vec<Pk> = users_groupes::Entity::find()
        .select_only()
        .column(users_groupes::Column::UserId)
        .filter(users_groupes::Column::GroupeId.eq(groupe_id))
        .into_tuple()
        .all(db)
        .await?;
    let ranked = user::Entity::find()
        .filter(user::Column::Id.is_in(members))
        .filter(
            Condition::any()
                .add(user::Column::IsAdmin.eq(true))
                .add(user::Column::IsDirigeant.eq(true))
                .add(user::Column::IsSuperuser.eq(true)),
        )
        .one(db)
        .await?;
    if ranked.is_some() {
        tracing::info!("group transfer refused: ranked members for an admin");
        return Err(DbErr::Custom(
            t("admin.transfer.ranked_members").into_owned(),
        ));
    }
    let grid = groupes_droits::Entity::find()
        .filter(groupes_droits::Column::UserId.eq(to))
        .all(db)
        .await?;
    let kept = as_rows(&group_rows_within(groupe_id, rows.clone(), &grid));
    let lines = lines(&rows, &kept);
    let losses = shown_of(&lines);
    Ok(Plan {
        group,
        from,
        to: receiver,
        lines,
        losses,
        rows,
        grid: Some(grid),
    })
}

/// The outcome of a confirmed transfer.
pub(crate) enum Transfer {
    Done,
    /// The rights at stake changed since the dirigeant saw them: nothing
    /// written, shown again.
    Changed,
}

/// Hands group `groupe_id` to `to`, as `actor`, if the rights at stake are
/// exactly `shown` (what the confirmation page listed). The lines of `keep`,
/// by their place in that list, go to the receiver's grid; the group keeps what that grid then covers,
/// stored. One transaction: the receiver's grid, the group's rows, the
/// responsable (`repris_par` emptied: the group is handed over), their
/// history rows. A right to keep that isn't at stake, or a full right on
/// the groups, refuses the whole.
pub(crate) async fn transfer_group(
    actor: &CurrentUser,
    groupe_id: i32,
    to: Pk,
    shown: &str,
    keep: &std::collections::HashSet<usize>,
    db: &ADb,
) -> Result<Transfer, DbErr> {
    let txn = db.begin().await?;
    let plan = plan(actor, groupe_id, to, &txn).await?;
    if plan.losses != shown {
        return Ok(Transfer::Changed);
    }
    if keep
        .iter()
        .any(|&i| plan.lines.get(i).is_none_or(|l| !l.keepable))
    {
        return Err(refused(RefusalReason::NoRight));
    }
    let keep: Vec<Line> = keep.iter().map(|&i| plan.lines[i].clone()).collect();

    // The receiver qualifies again in the write itself, and the group still
    // answers to whom the plan read.
    let qualifies = Query::select()
        .expr(Expr::val(1))
        .from(user::Entity)
        .cond_where(may_receive().add(user::Column::Id.eq(to)))
        .to_owned();
    let written = groupe::Entity::update_many()
        .col_expr(groupe::Column::ResponsableId, Expr::value(to))
        .col_expr(groupe::Column::ReprisPar, Expr::value(Option::<Pk>::None))
        .filter(groupe::Column::Id.eq(groupe_id))
        .filter(groupe::Column::ResponsableId.eq(plan.group.responsable_id))
        .filter(Expr::exists(qualifies))
        .exec(&txn)
        .await?;
    if written.rows_affected != 1 {
        return Err(unsuitable("changed during the transfer"));
    }

    let mut summary = serde_json::json!({
        "responsable": { "old": plan.from, "new": plan.to.username }
    });
    if let Some(grid) = plan.grid {
        let grid_after = widened(&grid, to, &keep);
        if !keep.is_empty() {
            groupes_droits::Entity::delete_many()
                .filter(groupes_droits::Column::UserId.eq(to))
                .exec(&txn)
                .await?;
            groupes_droits::Entity::insert_many(grid_after.iter().map(fresh))
                .exec(&txn)
                .await?;
            let rights = serde_json::json!({
                "droits": {
                    "old": describe(&from_rows(&grid)),
                    "new": describe(&from_rows(&grid_after)),
                }
            });
            log_action(
                &txn,
                ActionLog {
                    user_id: actor.id,
                    username: &actor.username,
                    resource_key: resource_key::USERS,
                    object_pk: &to.to_string(),
                    object_repr: &plan.to.username,
                    action: "rights",
                    summary: Some(rights.to_string()),
                    batch_id: None,
                },
            )
            .await?;
        }
        let kept = group_rows_within(groupe_id, plan.rows.clone(), &grid_after);
        let before = describe(&from_rows(&plan.rows));
        let after = describe(&from_rows(&as_rows(&kept)));
        groupes_droits::Entity::delete_many()
            .filter(groupes_droits::Column::GroupeId.eq(groupe_id))
            .exec(&txn)
            .await?;
        if !kept.is_empty() {
            groupes_droits::Entity::insert_many(kept).exec(&txn).await?;
        }
        if after != before {
            summary["droits"] = serde_json::json!({ "old": before, "new": after });
        }
    }
    log_action(
        &txn,
        ActionLog {
            user_id: actor.id,
            username: &actor.username,
            resource_key: resource_key::GROUPES,
            object_pk: &groupe_id.to_string(),
            object_repr: &plan.group.nom,
            action: "transfer",
            summary: Some(summary.to_string()),
            batch_id: None,
        },
    )
    .await?;
    txn.commit().await?;
    Ok(Transfer::Done)
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use crate::auth::gate::test_support::fixtures::{as_rank, groups_db, pk};
    use sea_orm::{ActiveModelTrait, Set};
    use std::collections::HashSet;

    /// A row of `holder` (a group, else an account's grid) on `key`/`colonne`.
    async fn row(
        db: &ADb,
        group: Option<i32>,
        user: Option<Pk>,
        key: &str,
        colonne: &str,
        f: [bool; 7],
    ) {
        groupes_droits::ActiveModel {
            groupe_id: Set(group),
            user_id: Set(user),
            resource_key: Set(key.into()),
            colonne: Set(colonne.into()),
            can_create: Set(f[0]),
            can_read: Set(f[1]),
            can_read_own: Set(f[2]),
            can_update: Set(f[3]),
            can_update_own: Set(f[4]),
            can_delete: Set(f[5]),
            can_delete_own: Set(f[6]),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();
    }

    const ALL: [bool; 7] = [true; 7];
    /// A column's rights: read, create, update (the table's other flags unused).
    const COLUMN: [bool; 7] = [true, true, false, true, false, false, false];

    /// Group 1 (the dirigeant's, `pk(32)`) holds every table right on
    /// `notes`, every right on its column `a`, and create + read on the groups.
    async fn setup() -> ADb {
        let db = groups_db(pk(32)).await;
        row(&db, Some(1), None, "notes", "", ALL).await;
        row(&db, Some(1), None, "notes", "a", COLUMN).await;
        row(
            &db,
            Some(1),
            None,
            resource_key::GROUPES,
            "",
            [true, true, false, false, false, false, false],
        )
        .await;
        db
    }

    async fn rows_of(db: &ADb, group: Option<i32>, user: Option<Pk>) -> Vec<groupes_droits::Model> {
        let mut q = groupes_droits::Entity::find();
        q = match (group, user) {
            (Some(g), _) => q.filter(groupes_droits::Column::GroupeId.eq(g)),
            (_, Some(u)) => q.filter(groupes_droits::Column::UserId.eq(u)),
            _ => q,
        };
        q.all(db).await.unwrap()
    }

    fn rights(rows: &[groupes_droits::Model], key: &str, colonne: &str) -> Vec<&'static str> {
        let Some(r) = rows
            .iter()
            .find(|r| r.resource_key == key && r.colonne == colonne)
        else {
            return Vec::new();
        };
        [
            (r.can_create, "create"),
            (r.can_read, "read"),
            (r.can_read_own, "read_own"),
            (r.can_update, "update"),
            (r.can_update_own, "update_own"),
            (r.can_delete, "delete"),
            (r.can_delete_own, "delete_own"),
        ]
        .into_iter()
        .filter_map(|(on, name)| on.then_some(name))
        .collect()
    }

    async fn responsable(db: &ADb) -> Pk {
        groupe::Entity::find_by_id(1)
            .one(db)
            .await
            .unwrap()
            .unwrap()
            .responsable_id
    }

    /// Every right the admin lacks is at stake, table and column, each named
    /// once; a full right on the groups is at stake but can't be kept.
    #[tokio::test]
    async fn every_right_the_receiver_lacks_is_at_stake() {
        let db = setup().await;
        let plan = plan(&as_rank(32, Rang::Dirigeant), 1, pk(33), &db)
            .await
            .unwrap();
        let named: Vec<String> = plan
            .lines
            .iter()
            .map(|l| {
                format!(
                    "{}.{}.{}:{}",
                    l.resource_key, l.colonne, l.right, l.keepable
                )
            })
            .collect();
        assert_eq!(
            named,
            [
                "groupes..create:true",
                "groupes..read:false",
                "notes..create:true",
                "notes..read:true",
                "notes..read_own:true",
                "notes..update:true",
                "notes..update_own:true",
                "notes..delete:true",
                "notes..delete_own:true",
                "notes.a.read:true",
                "notes.a.create:true",
                "notes.a.update:true",
            ]
        );
        assert_eq!(plan.losses, shown_of(&plan.lines));
        assert!(plan.losses.contains("notes.a.update"));
    }

    /// Every right kept goes to the admin's grid and stays with the group;
    /// the full right on the groups is lost all the same.
    #[tokio::test]
    async fn the_rights_kept_go_to_the_receiver_and_stay() {
        let db = setup().await;
        let boss = as_rank(32, Rang::Dirigeant);
        let plan = plan(&boss, 1, pk(33), &db).await.unwrap();
        let keep: HashSet<usize> = (0..plan.lines.len())
            .filter(|&i| plan.lines[i].keepable)
            .collect();
        let done = transfer_group(&boss, 1, pk(33), &plan.losses, &keep, &db)
            .await
            .unwrap();
        assert!(matches!(done, Transfer::Done));
        assert_eq!(responsable(&db).await, pk(33));

        let group = rows_of(&db, Some(1), None).await;
        assert_eq!(rights(&group, "notes", "").len(), 7);
        assert_eq!(rights(&group, "notes", "a"), ["create", "read", "update"]);
        assert_eq!(rights(&group, resource_key::GROUPES, ""), ["create"]);
        let grid = rows_of(&db, None, Some(pk(33))).await;
        assert_eq!(rights(&grid, "notes", "").len(), 7);
        assert_eq!(rights(&grid, "notes", "a"), ["create", "read", "update"]);
        assert_eq!(rights(&grid, resource_key::GROUPES, ""), ["create"]);
    }

    /// Nothing kept: the group loses all the admin lacks, his grid unchanged.
    #[tokio::test]
    async fn nothing_kept_loses_everything_at_stake() {
        let db = setup().await;
        row(
            &db,
            None,
            Some(pk(33)),
            "notes",
            "",
            [false, true, false, false, false, false, false],
        )
        .await;
        let boss = as_rank(32, Rang::Dirigeant);
        let plan = plan(&boss, 1, pk(33), &db).await.unwrap();
        // What his grid covers is not at stake: `read` and `read_own` on the
        // table, nothing else, the column's rights included.
        let at_stake = |key: &str, colonne: &str, right: &str| {
            plan.lines
                .iter()
                .any(|l| l.resource_key == key && l.colonne == colonne && l.right == right)
        };
        assert!(!at_stake("notes", "", "read") && !at_stake("notes", "", "read_own"));
        assert!(at_stake("notes", "", "update") && at_stake("notes", "a", "read"));
        assert_eq!(plan.lines.len(), 10);
        transfer_group(&boss, 1, pk(33), &plan.losses, &HashSet::new(), &db)
            .await
            .unwrap();
        let group = rows_of(&db, Some(1), None).await;
        // `read` covers `read_own`: both stay.
        assert_eq!(
            rights(&group, "notes", ""),
            ["read", "read_own"],
            "what he covered stays"
        );
        assert!(rights(&group, resource_key::GROUPES, "").is_empty());
        let grid = rows_of(&db, None, Some(pk(33))).await;
        assert_eq!(rights(&grid, "notes", ""), ["read"]);
        assert_eq!(grid.len(), 1);
    }

    /// The losses shown for another receiver don't confirm this one; a line
    /// not at stake, or not keepable, refuses the whole.
    #[tokio::test]
    async fn a_transfer_goes_through_for_its_own_lines_only() {
        let db = setup().await;
        let boss = as_rank(32, Rang::Dirigeant);
        let other = plan(&boss, 1, pk(30), &db).await.unwrap();
        assert!(other.lines.is_empty(), "the superuser covers everything");
        let changed = transfer_group(&boss, 1, pk(33), &other.losses, &HashSet::new(), &db)
            .await
            .unwrap();
        assert!(matches!(changed, Transfer::Changed));
        assert_eq!(responsable(&db).await, pk(32));

        let plan = plan(&boss, 1, pk(33), &db).await.unwrap();
        let locked = plan.lines.iter().position(|l| !l.keepable).unwrap();
        for keep in [HashSet::from([locked]), HashSet::from([plan.lines.len()])] {
            assert!(
                transfer_group(&boss, 1, pk(33), &plan.losses, &keep, &db)
                    .await
                    .is_err()
            );
        }
        assert_eq!(responsable(&db).await, pk(32));
    }

    /// Below dirigeant nobody transfers; nor to the responsable in place;
    /// to a superuser the group stays whole.
    #[tokio::test]
    async fn a_transfer_needs_a_dirigeant_and_a_new_receiver() {
        let db = setup().await;
        assert!(
            plan(&as_rank(33, Rang::Admin), 1, pk(30), &db)
                .await
                .is_err()
        );
        let boss = as_rank(32, Rang::Dirigeant);
        assert!(plan(&boss, 1, pk(32), &db).await.is_err());
        transfer_group(&boss, 1, pk(30), "", &HashSet::new(), &db)
            .await
            .unwrap();
        assert_eq!(responsable(&db).await, pk(30));
        assert_eq!(
            rights(&rows_of(&db, Some(1), None).await, "notes", "").len(),
            7
        );
    }
}
