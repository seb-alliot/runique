//! The gate's decision on one row, without the database: the async gate
//! looks up what it needs (owner), then asks here.
use super::operation::AdminOperation;
use super::refusal::{Refusal, RefusalReason};
use super::rights::CrudAction;
use crate::auth::rang::Rang;
use sea_orm::Condition;

/// Who owns the targeted row, as the gate read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
    /// No row (create) or no owner column: `_own` rights never open anything.
    NotApplicable,
    Owner,
    NotOwner,
}

/// Group rights for the operation `O`: the full right, or the `_own` right
/// on a row the user owns.
pub fn rights_allow<O: AdminOperation>(
    rights: &CrudAction,
    ownership: Ownership,
) -> Result<(), Refusal> {
    if rights.has(O::RIGHT) {
        return Ok(());
    }
    match O::RIGHT_OWN {
        Some(own) if rights.has(own) => match ownership {
            Ownership::Owner => Ok(()),
            Ownership::NotOwner | Ownership::NotApplicable => {
                Err(Refusal::new(RefusalReason::NotOwner))
            }
        },
        _ => Err(Refusal::new(RefusalReason::NoRight)),
    }
}

/// The accounts `rang` may reach through `O`, as a condition for the write:
/// nothing to add if `O` doesn't protect accounts, refused if the rank
/// manages none. The target's rank is never read beforehand: the condition
/// in the write's WHERE is the check.
pub fn rank_allows<O: AdminOperation>(rang: Rang) -> Result<Option<Condition>, Refusal> {
    if !O::PROTECTS_RANK {
        return Ok(None);
    }
    rang.cibles_gerables()
        .map(Some)
        .ok_or_else(|| Refusal::new(RefusalReason::RankTooLow))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::refonte_admin::gate::operation::{
        Create, Delete, List, ResetPassword, Update, View,
    };

    fn rights(f: impl FnOnce(&mut CrudAction)) -> CrudAction {
        let mut r = CrudAction::default();
        f(&mut r);
        r
    }

    fn reason(r: Result<(), Refusal>) -> Option<RefusalReason> {
        r.err().map(|e| *e.reason())
    }

    #[test]
    fn the_full_right_opens_whoever_owns_the_row() {
        let r = rights(|r| r.can_update = true);
        for o in [
            Ownership::Owner,
            Ownership::NotOwner,
            Ownership::NotApplicable,
        ] {
            assert!(rights_allow::<Update>(&r, o).is_ok(), "{o:?}");
        }
    }

    #[test]
    fn the_own_right_opens_only_the_users_own_row() {
        let r = rights(|r| r.can_update_own = true);
        assert!(rights_allow::<Update>(&r, Ownership::Owner).is_ok());
        assert_eq!(
            reason(rights_allow::<Update>(&r, Ownership::NotOwner)),
            Some(RefusalReason::NotOwner)
        );
        assert_eq!(
            reason(rights_allow::<Update>(&r, Ownership::NotApplicable)),
            Some(RefusalReason::NotOwner),
            "no owner column: _own never opens"
        );
    }

    #[test]
    fn an_own_right_of_another_operation_opens_nothing() {
        let r = rights(|r| r.can_delete_own = true);
        assert_eq!(
            reason(rights_allow::<Update>(&r, Ownership::Owner)),
            Some(RefusalReason::NoRight)
        );
        assert!(rights_allow::<Delete>(&r, Ownership::Owner).is_ok());
    }

    #[test]
    fn create_and_reset_password_have_no_own_right() {
        let r = rights(|r| {
            r.can_update_own = true;
            r.can_delete_own = true;
            r.can_read_own = true;
        });
        assert_eq!(
            reason(rights_allow::<Create>(&r, Ownership::Owner)),
            Some(RefusalReason::NoRight)
        );
        assert_eq!(
            reason(rights_allow::<ResetPassword>(&r, Ownership::Owner)),
            Some(RefusalReason::NoRight)
        );
    }

    #[test]
    fn each_operation_asks_its_own_right() {
        let read = rights(|r| r.can_read = true);
        assert!(rights_allow::<List>(&read, Ownership::NotApplicable).is_ok());
        assert!(rights_allow::<View>(&read, Ownership::NotOwner).is_ok());
        assert!(rights_allow::<Update>(&read, Ownership::Owner).is_err());
        assert!(rights_allow::<Create>(&read, Ownership::NotApplicable).is_err());

        let create = rights(|r| r.can_create = true);
        assert!(rights_allow::<Create>(&create, Ownership::NotApplicable).is_ok());
        assert!(rights_allow::<View>(&create, Ownership::Owner).is_err());

        let update = rights(|r| r.can_update = true);
        assert!(rights_allow::<ResetPassword>(&update, Ownership::NotOwner).is_ok());
        assert!(rights_allow::<Delete>(&update, Ownership::Owner).is_err());
    }

    #[test]
    fn an_account_without_rank_manages_no_account() {
        for refused in [
            rank_allows::<Update>(Rang::Lambda),
            rank_allows::<Delete>(Rang::Lambda),
            rank_allows::<ResetPassword>(Rang::Lambda),
        ] {
            assert_eq!(
                refused.err().map(|r| *r.reason()),
                Some(RefusalReason::RankTooLow)
            );
        }
        // The other side: reading and creating protect no account.
        for open in [
            rank_allows::<View>(Rang::Lambda),
            rank_allows::<List>(Rang::Lambda),
            rank_allows::<Create>(Rang::Lambda),
        ] {
            assert!(matches!(open, Ok(None)));
        }
    }

    /// Each rank excludes its own flag and every higher one: never a peer,
    /// never oneself, never above.
    #[test]
    fn each_rank_reaches_only_lower_ranks() {
        use sea_orm::{DbBackend, EntityTrait, QueryFilter, QueryTrait};
        // The WHERE alone: the SELECT lists every column, flags included.
        let excluded = |rang: Rang| {
            let condition = rank_allows::<Update>(rang).unwrap().unwrap();
            let sql = crate::auth::user::Entity::find()
                .filter(condition)
                .build(DbBackend::Sqlite)
                .to_string();
            let flags = ["is_superuser", "is_dirigeant", "is_admin"];
            let condition = sql.split_once("WHERE").map_or("", |(_, w)| w).to_string();
            flags.map(|flag| condition.contains(flag))
        };
        assert_eq!(excluded(Rang::Admin), [true, true, true]);
        assert_eq!(excluded(Rang::Dirigeant), [true, true, false]);
        assert_eq!(excluded(Rang::Superuser), [true, false, false]);
    }
}
