//! What the list and the detail of accounts show about each one: its rank and
//! its groups, read-only. Whoever may read the accounts sees them; changing
//! either goes through the gate.
use crate::{
    auth::{
        permissions::{group, rank::Rang, user_groups},
        user,
    },
    schema::pk::Pk,
    translation::t,
    utils::aliases::ADb,
};
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

/// The message that names `rang`.
pub(crate) fn rank_label(rang: Rang) -> &'static str {
    use runique_i18n::keys::admin::rank::label;
    match rang {
        Rang::Lambda => label::LAMBDA,
        Rang::Admin => label::ADMIN,
        Rang::Dirigeant => label::DIRIGEANT,
        Rang::Superuser => label::SUPERUSER,
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
    let memberships = user_groups::Entity::find()
        .filter(user_groups::Column::UserId.is_in(ids.iter().copied()))
        .find_also_related(group::Entity)
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
            let rang = Rang::from(account);
            let rank = rank_key(rang);
            let mut names = groups.remove(&account.id).unwrap_or_default();
            names.sort_unstable();
            let summary = AccountSummary {
                rank,
                rank_label: t(rank_label(rang)).into_owned(),
                groups: names,
            };
            (account.id, summary)
        })
        .collect())
}

#[cfg(test)]
mod rank_label_tests {
    use super::*;

    const RANKS: [Rang; 4] = [Rang::Lambda, Rang::Admin, Rang::Dirigeant, Rang::Superuser];

    // Each rank has its own message, and that message exists: a key without
    // one would show as the key itself.
    #[test]
    fn each_rank_has_its_own_message() {
        let mut seen = std::collections::HashSet::new();
        for rang in RANKS {
            let key = rank_label(rang);
            assert!(seen.insert(key), "{rang:?} shares {key}");
            assert_ne!(t(key), key, "{rang:?}: no message for {key}");
        }
    }

    // The message follows the rank's name in forms and templates.
    #[test]
    fn the_message_of_a_rank_is_named_after_it() {
        for rang in RANKS {
            assert!(
                rank_label(rang).ends_with(&format!(".{}", rank_key(rang))),
                "{rang:?}: {}",
                rank_label(rang)
            );
        }
    }
}
