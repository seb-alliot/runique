//! Rank of an account in the administration hierarchy.
#[cfg(feature = "admin")]
use sea_orm::{ColumnTrait, Condition};
use serde::{Deserialize, Serialize};

#[cfg(feature = "admin")]
use crate::auth::user::Column;
use crate::auth::user::Model;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rang {
    Lambda,
    Admin,
    Dirigeant,
    Superuser,
}

impl From<&Model> for Rang {
    fn from(user: &Model) -> Self {
        match (user.is_superuser, user.is_dirigeant, user.is_admin) {
            (true, _, _) => Rang::Superuser,
            (_, true, _) => Rang::Dirigeant,
            (_, _, true) => Rang::Admin,
            _ => Rang::Lambda,
        }
    }
}

impl Rang {
    /// Accounts this rank may manage, to add to the WHERE of the write: only
    /// a strictly lower rank, so never one's own account. `None`: manages no
    /// account, refused before any write.
    #[cfg(feature = "admin")]
    pub(crate) fn cibles_gerables(self) -> Option<Condition> {
        let exclus: &[Column] = match self {
            Rang::Lambda => return None,
            Rang::Admin => &[Column::IsSuperuser, Column::IsDirigeant, Column::IsAdmin],
            Rang::Dirigeant => &[Column::IsSuperuser, Column::IsDirigeant],
            Rang::Superuser => &[Column::IsSuperuser],
        };
        Some(
            exclus
                .iter()
                .fold(Condition::all(), |cond, col| cond.add(col.eq(false))),
        )
    }
}
