//! What a user CAN do on a resource, read from their groups on every request.
use crate::auth::permissions::Permission;
use crate::auth::rang::Rang;
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
    /// The rights of `user` on the resource `key`: the superuser has them all
    /// and bypasses, the dirigeant has them all without bypassing, an admin
    /// has what the dirigeant set on his account, a lambda the union of his
    /// groups, never on a framework table.
    pub fn resolve(user: &CurrentUser, key: &str, framework_table: bool) -> Self {
        match user.rang {
            Rang::Superuser => Self::everything(),
            Rang::Dirigeant => Self {
                is_superuser: false,
                ..Self::everything()
            },
            Rang::Admin => user.droit_for(key).map_or_else(Self::default, Self::from),
            Rang::Lambda if framework_table => Self::default(),
            Rang::Lambda => user
                .permission_for(key)
                .as_ref()
                .map_or_else(Self::default, Self::from),
        }
    }
    fn everything() -> Self {
        Self {
            can_create: true,
            can_read: true,
            can_read_own: true,
            can_update: true,
            can_update_own: true,
            can_delete: true,
            can_delete_own: true,
            is_superuser: true,
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

impl From<&Permission> for CrudAction {
    fn from(p: &Permission) -> Self {
        Self {
            can_create: p.can_create,
            can_read: p.can_read,
            can_read_own: p.can_read_own,
            can_update: p.can_update,
            can_update_own: p.can_update_own,
            can_delete: p.can_delete,
            can_delete_own: p.can_delete_own,
            is_superuser: false,
        }
    }
}
/// The columns a user may read on a model, bayond its primary key (always readable).
/// Checked by the gate only under the table's read right, so a column is read with rights AND its own row:
/// nothing added later shows until it is set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadableColumns {
    /// The superuser and the Dirigeant: every column (the builder still keeps the secret on out)
    All,
    /// Everyone else: these columns only.
    Only(Vec<String>),
}

impl ReadableColumns {
    pub fn resolve(user: &CurrentUser, key: &str, framework_table: bool) -> Self {
        match user.rang {
            Rang::Superuser | Rang::Dirigeant => Self::All,
            Rang::Admin => Self::ticked(user.droit_for(key)),
            Rang::Lambda if framework_table => Self::Only(Vec::new()),
            Rang::Lambda => Self::ticked(user.permission_for(key).as_ref()),
        }
    }

    fn ticked(right: Option<&Permission>) -> Self {
        Self::Only(right.map_or_else(Vec::new, |p| {
            p.colonnes
                .iter()
                .filter(|c| c.can_read)
                .map(|c| c.colonne.clone())
                .collect()
        }))
    }

    pub fn allows(&self, column: &str) -> bool {
        match self {
            Self::All => true,
            Self::Only(cols) => cols.iter().any(|c| c == column),
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
    use crate::auth::permissions::Groupe;
    use crate::auth::session::CurrentUser;

    const ALL: [Crud; 7] = [
        Crud::Create,
        Crud::Read,
        Crud::ReadOwn,
        Crud::Update,
        Crud::UpdateOwn,
        Crud::Delete,
        Crud::DeleteOwn,
    ];

    fn perm(key: &str, set: impl FnOnce(&mut Permission)) -> Permission {
        let mut p = Permission::zeroed(key.into());
        set(&mut p);
        p
    }

    /// One group per entry of `groupes`.
    fn user(rang: Rang, groupes: Vec<Vec<Permission>>, droits: Vec<Permission>) -> CurrentUser {
        CurrentUser {
            id: Pk::default(),
            username: "u".into(),
            rang,
            groupes: groupes
                .into_iter()
                .enumerate()
                .map(|(i, permissions)| Groupe {
                    id: i as i32,
                    nom: format!("g{i}"),
                    is_staff: false,
                    permissions,
                })
                .collect(),
            droits,
        }
    }

    #[test]
    fn a_superuser_has_every_right_and_bypasses_a_lambda_without_group_none() {
        for framework in [false, true] {
            let su =
                CrudAction::resolve(&user(Rang::Superuser, vec![], vec![]), "plats", framework);
            let nobody =
                CrudAction::resolve(&user(Rang::Lambda, vec![], vec![]), "plats", framework);
            for right in ALL {
                assert!(su.has(right), "{right:?}");
                assert!(!nobody.has(right), "{right:?}");
            }
            assert!(su.is_superuser);
            assert!(!nobody.is_superuser);
        }
    }

    #[test]
    fn a_dirigeant_has_every_right_without_bypassing() {
        for framework in [false, true] {
            let d = CrudAction::resolve(&user(Rang::Dirigeant, vec![], vec![]), "users", framework);
            for right in ALL {
                assert!(d.has(right), "{right:?}");
            }
            assert!(!d.is_superuser);
        }
    }

    #[test]
    fn an_admin_has_only_what_his_account_holds() {
        let admin = user(
            Rang::Admin,
            vec![],
            vec![
                perm("plats", |p| p.can_read = true),
                perm("users", |p| p.can_update = true),
            ],
        );
        let plats = CrudAction::resolve(&admin, "plats", false);
        assert!(plats.has(Crud::Read));
        assert!(!plats.has(Crud::Create) && !plats.has(Crud::Update) && !plats.has(Crud::Delete));
        assert_eq!(
            CrudAction::resolve(&admin, "menus", false),
            CrudAction::default()
        );

        let users = CrudAction::resolve(&admin, "users", true);
        assert!(
            users.has(Crud::Update),
            "a framework table, when the dirigeant set it"
        );
        assert!(!users.has(Crud::Read));
        assert_eq!(
            CrudAction::resolve(&admin, "groupes", true),
            CrudAction::default(),
            "nothing on a framework table not set"
        );
    }

    #[test]
    fn an_admin_ignores_his_groups_and_a_lambda_his_account_rows() {
        let admin = user(
            Rang::Admin,
            vec![vec![perm("plats", |p| p.can_delete = true)]],
            vec![],
        );
        assert_eq!(
            CrudAction::resolve(&admin, "plats", false),
            CrudAction::default()
        );

        let demoted = user(
            Rang::Lambda,
            vec![],
            vec![perm("plats", |p| p.can_read = true)],
        );
        assert_eq!(
            CrudAction::resolve(&demoted, "plats", false),
            CrudAction::default()
        );
    }

    #[test]
    fn a_lambda_has_the_union_of_his_groups_never_on_a_framework_table() {
        let lambda = user(
            Rang::Lambda,
            vec![
                vec![
                    perm("plats", |p| p.can_read = true),
                    perm("users", |p| p.can_read = true),
                ],
                vec![perm("plats", |p| p.can_update = true)],
            ],
            vec![],
        );
        let plats = CrudAction::resolve(&lambda, "plats", false);
        assert!(plats.has(Crud::Read) && plats.has(Crud::Update));
        assert!(!plats.has(Crud::Create) && !plats.has(Crud::Delete));
        assert_eq!(
            CrudAction::resolve(&lambda, "users", true),
            CrudAction::default()
        );
    }

    #[test]
    fn read_own_comes_from_the_rows() {
        let lambda = user(
            Rang::Lambda,
            vec![vec![perm("plats", |p| p.can_read_own = true)]],
            vec![],
        );
        let plats = CrudAction::resolve(&lambda, "plats", false);
        assert!(plats.has(Crud::ReadOwn));
        assert!(!plats.has(Crud::Read));
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
    #[test]
    fn readable_columns_follow_the_rank_and_the_ticked_columns() {
        use crate::auth::permissions::ColumnRight;
        let col = |name: &str, read: bool| ColumnRight {
            colonne: name.into(),
            can_read: read,
            can_create: false,
            can_update: false,
        };
        let plats = |cols: Vec<ColumnRight>| {
            perm("plats", |p| {
                p.can_read = true;
                p.colonnes = cols;
            })
        };

        let lambda = user(
            Rang::Lambda,
            vec![
                vec![plats(vec![col("titre", true)])],
                vec![plats(vec![col("prix", true), col("stock", false)])],
            ],
            vec![],
        );
        let cols = ReadableColumns::resolve(&lambda, "plats", false);
        assert!(
            cols.allows("titre") && cols.allows("prix"),
            "the union of the groups"
        );
        assert!(!cols.allows("stock") && !cols.allows("note"));
        assert_eq!(
            ReadableColumns::resolve(&lambda, "plats", true),
            ReadableColumns::Only(vec![])
        );

        let admin = user(Rang::Admin, vec![], vec![plats(vec![col("titre", true)])]);
        let cols = ReadableColumns::resolve(&admin, "plats", false);
        assert!(cols.allows("titre") && !cols.allows("prix"));

        let dirigeant = user(Rang::Dirigeant, vec![], vec![]);
        assert_eq!(
            ReadableColumns::resolve(&dirigeant, "plats", true),
            ReadableColumns::All
        );
    }
}
