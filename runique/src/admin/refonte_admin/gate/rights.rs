//! What a user CAN do on a resource, read from their groups on every request.
use crate::auth::session::CurrentUser;
use crate::utils::pk::Pk;
use sea_orm::EntityTrait;

/// The user's rights on one resource. A superuser has every right.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CrudAction {
    pub can_create: bool,
    pub can_read: bool,
    /// Not stored yet: needs its own column in `eihwaz_groupes_droits`.
    /// Until then it is only ever `true` for a superuser.
    pub can_read_own: bool,
    pub can_update: bool,
    pub can_update_own: bool,
    pub can_delete: bool,
    pub can_delete_own: bool,
    pub is_superuser: bool,
}

/// One right, as an operation asks for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crud {
    Create,
    Read,
    ReadOwn,
    Update,
    UpdateOwn,
    Delete,
    DeleteOwn,
}

impl CrudAction {
    /// The rights of `user` on the resource `key`.
    pub fn resolve(user: &CurrentUser, key: &str) -> Self {
        if user.is_superuser {
            return Self {
                can_create: true,
                can_read: true,
                can_read_own: true,
                can_update: true,
                can_update_own: true,
                can_delete: true,
                can_delete_own: true,
                is_superuser: true,
            };
        }
        match user.permission_for(key) {
            Some(p) => Self {
                can_create: p.can_create,
                can_read: p.can_read,
                can_read_own: false,
                can_update: p.can_update,
                can_update_own: p.can_update_own,
                can_delete: p.can_delete,
                can_delete_own: p.can_delete_own,
                is_superuser: false,
            },
            None => Self::default(),
        }
    }

    pub fn has(&self, right: Crud) -> bool {
        match right {
            Crud::Create => self.can_create,
            Crud::Read => self.can_read,
            Crud::ReadOwn => self.can_read_own,
            Crud::Update => self.can_update,
            Crud::UpdateOwn => self.can_update_own,
            Crud::Delete => self.can_delete,
            Crud::DeleteOwn => self.can_delete_own,
        }
    }
}

/// Which rows a query on several rows (list, group action) may reach.
pub enum Scope<E: EntityTrait> {
    /// Full right: every row (still narrowed by the resource's business filter).
    All,
    /// `_own` right only: the query is restricted to `column = user`.
    Own(E::Column, Pk),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::session::CurrentUser;

    fn user(is_superuser: bool) -> CurrentUser {
        CurrentUser {
            id: Pk::default(),
            username: "u".into(),
            is_staff: true,
            is_superuser,
            groupes: vec![],
        }
    }

    #[test]
    fn a_superuser_has_every_right_and_a_user_without_group_none() {
        let su = CrudAction::resolve(&user(true), "plats");
        let nobody = CrudAction::resolve(&user(false), "plats");
        for right in [
            Crud::Create,
            Crud::Read,
            Crud::ReadOwn,
            Crud::Update,
            Crud::UpdateOwn,
            Crud::Delete,
            Crud::DeleteOwn,
        ] {
            assert!(su.has(right), "{right:?}");
            assert!(!nobody.has(right), "{right:?}");
        }
        assert!(su.is_superuser);
        assert!(!nobody.is_superuser);
    }

    #[test]
    fn has_reads_the_matching_flag_only() {
        let only_update_own = CrudAction {
            can_update_own: true,
            ..Default::default()
        };
        assert!(only_update_own.has(Crud::UpdateOwn));
        assert!(!only_update_own.has(Crud::Update));
        assert!(!only_update_own.has(Crud::DeleteOwn));
        assert!(!only_update_own.has(Crud::ReadOwn));
    }
}
