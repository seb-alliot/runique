//! The gate's decision on one row, without the database: the async gate
//! looks up what it needs (owner, target account), then asks here.
use super::operation::AdminOperation;
use super::refusal::{Refusal, RefusalReason};
use super::rights::CrudAction;

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

/// A non-superuser can't target a superuser's account through an operation
/// that protects it.
pub fn superuser_allows<O: AdminOperation>(
    rights: &CrudAction,
    target_is_superuser: bool,
) -> Result<(), Refusal> {
    if O::PROTECTS_SUPERUSER && target_is_superuser && !rights.is_superuser {
        return Err(Refusal::new(RefusalReason::ProtectedSuperuser));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::gate::operation::{Create, Delete, List, ResetPassword, Update, View};

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
    fn a_superuser_account_is_out_of_reach_of_a_non_superuser() {
        let staff = rights(|r| {
            r.can_update = true;
            r.can_delete = true;
        });
        for refused in [
            superuser_allows::<Update>(&staff, true),
            superuser_allows::<Delete>(&staff, true),
            superuser_allows::<ResetPassword>(&staff, true),
        ] {
            assert_eq!(reason(refused), Some(RefusalReason::ProtectedSuperuser));
        }
        // The other sides: reading stays open, an ordinary account is reachable,
        // a superuser reaches another superuser through this flag.
        assert!(superuser_allows::<View>(&staff, true).is_ok());
        assert!(superuser_allows::<Update>(&staff, false).is_ok());
        let su = rights(|r| r.is_superuser = true);
        assert!(superuser_allows::<Update>(&su, true).is_ok());
    }
}
