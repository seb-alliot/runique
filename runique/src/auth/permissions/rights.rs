//! What a user CAN do on a resource: the rights the admin's gate and the
//! developer's own views both read.
use crate::auth::current_user::CurrentUser;
use crate::auth::permissions::rang::Rang;
use crate::auth::permissions::{ColumnRight, Permission};
use crate::utils::constante::resource_key::FRAMEWORK_KEYS;
use sea_orm::{EntityTrait, IdenStatic, Iterable, PrimaryKeyToColumn};

/// Everything a user may do on one resource, decided once by the rank: the
/// table's rights and, column by column, read / create / update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rights {
    pub crud: CrudAction,
    pub columns: ColumnRights,
}

/// Per column: read, create, update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnRights {
    /// The superuser and the dirigeant: every column.
    All,
    /// Everyone else: the columns ticked, nothing added later until it is.
    Only(Vec<ColumnRight>),
}

impl CurrentUser {
    /// What this user may do on `key`. The rank alone picks the source: the
    /// admin's own grid, the lambda's groups, never a group on a framework
    /// table. The only way to ask; the admin's gate asks it too.
    #[must_use]
    pub fn rights_on(&self, key: &str) -> Rights {
        match self.rang {
            Rang::Superuser => Rights::everything(true),
            Rang::Dirigeant => Rights::everything(false),
            Rang::Admin => Rights::granted(self.droits.iter().find(|p| p.resource_key == key)),
            Rang::Lambda if FRAMEWORK_KEYS.contains(&key) => Rights::granted(None),
            Rang::Lambda => Rights::granted(self.groups_union(key).as_ref()),
        }
    }
}

impl Rights {
    fn everything(bypass: bool) -> Self {
        Self {
            crud: CrudAction {
                is_superuser: bypass,
                ..CrudAction::everything()
            },
            columns: ColumnRights::All,
        }
    }

    fn granted(right: Option<&Permission>) -> Self {
        match right {
            Some(p) => Self {
                crud: CrudAction::from(p),
                columns: ColumnRights::Only(p.colonnes.clone()),
            },
            None => Self {
                crud: CrudAction::default(),
                columns: ColumnRights::Only(Vec::new()),
            },
        }
    }
}

impl ColumnRights {
    pub fn readable(&self) -> ReadableColumns {
        match self {
            Self::All => ReadableColumns::All,
            Self::Only(cols) => ReadableColumns::Only(
                cols.iter()
                    .filter(|c| c.can_read)
                    .map(|c| c.colonne.clone())
                    .collect(),
            ),
        }
    }

    pub fn can_create(&self, column: &str) -> bool {
        self.ticked(column, |c| c.can_create)
    }

    pub fn can_update(&self, column: &str) -> bool {
        self.ticked(column, |c| c.can_update)
    }

    fn ticked(&self, column: &str, right: impl Fn(&ColumnRight) -> bool) -> bool {
        match self {
            Self::All => true,
            Self::Only(cols) => cols.iter().any(|c| c.colonne == column && right(c)),
        }
    }
}

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
    /// The table's rights of `user` on `key`, as [`CurrentUser::rights_on`]
    /// decides them.
    pub fn resolve(user: &CurrentUser, key: &str) -> Self {
        user.rights_on(key).crud
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
    /// The columns `user` may read on `key`, as [`CurrentUser::rights_on`]
    /// decides them.
    pub fn resolve(user: &CurrentUser, key: &str) -> Self {
        user.rights_on(key).columns.readable()
    }

    pub fn allows(&self, column: &str) -> bool {
        match self {
            Self::All => true,
            Self::Only(cols) => cols.iter().any(|c| c == column),
        }
    }

    /// `cols` reduced to what may be read, in their order; the key always passes.
    pub fn preserve<E: EntityTrait>(&self, cols: &[E::Column]) -> Vec<E::Column> {
        cols.iter()
            .copied()
            .filter(|c| {
                E::PrimaryKey::iter().any(|k| k.into_column().as_str() == c.as_str())
                    || self.allows(c.as_str())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::current_user::CurrentUser;
    use crate::auth::permissions::Groupe;
    use crate::utils::pk::Pk;

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
        for key in ["plats", "users"] {
            let su = CrudAction::resolve(&user(Rang::Superuser, vec![], vec![]), key);
            let nobody = CrudAction::resolve(&user(Rang::Lambda, vec![], vec![]), key);
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
        for key in ["plats", "users"] {
            let d = CrudAction::resolve(&user(Rang::Dirigeant, vec![], vec![]), key);
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
        let plats = CrudAction::resolve(&admin, "plats");
        assert!(plats.has(Crud::Read));
        assert!(!plats.has(Crud::Create) && !plats.has(Crud::Update) && !plats.has(Crud::Delete));
        assert_eq!(CrudAction::resolve(&admin, "menus"), CrudAction::default());

        let users = CrudAction::resolve(&admin, "users");
        assert!(
            users.has(Crud::Update),
            "a framework table, when the dirigeant set it"
        );
        assert!(!users.has(Crud::Read));
        assert_eq!(
            CrudAction::resolve(&admin, "groupes"),
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
        assert_eq!(CrudAction::resolve(&admin, "plats"), CrudAction::default());

        let demoted = user(
            Rang::Lambda,
            vec![],
            vec![perm("plats", |p| p.can_read = true)],
        );
        assert_eq!(
            CrudAction::resolve(&demoted, "plats"),
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
        let plats = CrudAction::resolve(&lambda, "plats");
        assert!(plats.has(Crud::Read) && plats.has(Crud::Update));
        assert!(!plats.has(Crud::Create) && !plats.has(Crud::Delete));
        assert_eq!(CrudAction::resolve(&lambda, "users"), CrudAction::default());
    }

    #[test]
    fn read_own_comes_from_the_rows() {
        let lambda = user(
            Rang::Lambda,
            vec![vec![perm("plats", |p| p.can_read_own = true)]],
            vec![],
        );
        let plats = CrudAction::resolve(&lambda, "plats");
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

        let users = perm("users", |p| {
            p.can_read = true;
            p.colonnes = vec![col("email", true)];
        });
        let lambda = user(
            Rang::Lambda,
            vec![
                vec![plats(vec![col("titre", true)]), users],
                vec![plats(vec![col("prix", true), col("stock", false)])],
            ],
            vec![],
        );
        let cols = ReadableColumns::resolve(&lambda, "plats");
        assert!(
            cols.allows("titre") && cols.allows("prix"),
            "the union of the groups"
        );
        assert!(!cols.allows("stock") && !cols.allows("note"));
        assert_eq!(
            ReadableColumns::resolve(&lambda, "users"),
            ReadableColumns::Only(vec![]),
            "a group never opens a framework table"
        );

        let admin = user(Rang::Admin, vec![], vec![plats(vec![col("titre", true)])]);
        let cols = ReadableColumns::resolve(&admin, "plats");
        assert!(cols.allows("titre") && !cols.allows("prix"));

        let dirigeant = user(Rang::Dirigeant, vec![], vec![]);
        assert_eq!(
            ReadableColumns::resolve(&dirigeant, "users"),
            ReadableColumns::All
        );
    }

    /// Every framework table, not only the accounts: a group gets nothing on
    /// any of them, its rows notwithstanding.
    #[test]
    fn a_group_opens_no_framework_table() {
        for key in FRAMEWORK_KEYS {
            let lambda = user(
                Rang::Lambda,
                vec![vec![perm(key, |p| {
                    p.can_read = true;
                    p.can_update = true;
                })]],
                vec![],
            );
            assert_eq!(lambda.rights_on(key), Rights::granted(None), "{key}");
        }
        let lambda = user(
            Rang::Lambda,
            vec![vec![perm("plats", |p| p.can_read = true)]],
            vec![],
        );
        assert!(lambda.rights_on("plats").crud.has(Crud::Read));
    }

    #[test]
    fn column_writes_follow_their_own_ticks() {
        use crate::auth::permissions::ColumnRight;
        let admin = user(
            Rang::Admin,
            vec![],
            vec![perm("plats", |p| {
                p.can_update = true;
                p.colonnes = vec![
                    ColumnRight {
                        colonne: "titre".into(),
                        can_read: true,
                        can_create: false,
                        can_update: true,
                    },
                    ColumnRight {
                        colonne: "prix".into(),
                        can_read: true,
                        can_create: true,
                        can_update: false,
                    },
                ];
            })],
        );
        let cols = admin.rights_on("plats").columns;
        assert!(cols.can_update("titre") && !cols.can_create("titre"));
        assert!(cols.can_create("prix") && !cols.can_update("prix"));
        assert!(!cols.can_create("stock") && !cols.can_update("stock"));

        let dirigeant = user(Rang::Dirigeant, vec![], vec![]);
        let cols = dirigeant.rights_on("plats").columns;
        assert!(cols.can_create("stock") && cols.can_update("stock"));
    }
}
