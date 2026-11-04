//! What the list and the detail of accounts show about each one: its rank and
//! its groups, read-only. Whoever may read the accounts sees them; changing
//! either goes through the gate.
use crate::auth::permissions::rang::Rang;
use crate::auth::permissions::{groupe, users_groupes};
use crate::auth::user;
use crate::utils::aliases::ADb;
use crate::utils::pk::Pk;
use crate::utils::trad::t;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::Serialize;
use std::collections::HashMap;

/// An account's rank and groups, as a page shows them.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct AccountSummary {
    /// `lambda`, `admin`, `dirigeant` or `superuser`.
    pub rank: &'static str,
    /// The rank in the viewer's language.
    pub rank_label: String,
    /// Its groups' names, sorted.
    pub groups: Vec<String>,
}

/// The name of `rang` in forms, templates and translation keys.
pub(crate) fn rank_key(rang: Rang) -> &'static str {
    match rang {
        Rang::Lambda => "lambda",
        Rang::Admin => "admin",
        Rang::Dirigeant => "dirigeant",
        Rang::Superuser => "superuser",
    }
}

/// The summaries of `ids`, in two queries however many they are. An id without
/// an account has no entry.
pub(crate) async fn account_summaries(
    db: &ADb,
    ids: &[Pk],
) -> Result<HashMap<Pk, AccountSummary>, DbErr> {
    let accounts = user::Entity::find()
        .filter(user::Column::Id.is_in(ids.iter().copied()))
        .all(db)
        .await?;
    let mut groups: HashMap<Pk, Vec<String>> = HashMap::new();
    let memberships = users_groupes::Entity::find()
        .filter(users_groupes::Column::UserId.is_in(ids.iter().copied()))
        .find_also_related(groupe::Entity)
        .all(db)
        .await?;
    for (membership, group) in memberships {
        if let Some(group) = group {
            groups
                .entry(membership.user_id)
                .or_default()
                .push(group.nom);
        }
    }
    Ok(accounts
        .iter()
        .map(|account| {
            let rank = rank_key(Rang::from(account));
            let mut names = groups.remove(&account.id).unwrap_or_default();
            names.sort_unstable();
            let summary = AccountSummary {
                rank,
                rank_label: t(&format!("admin.rank.{rank}")).into_owned(),
                groups: names,
            };
            (account.id, summary)
        })
        .collect())
}
