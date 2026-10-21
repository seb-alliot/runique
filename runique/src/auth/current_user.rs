//! The signed-in account as one request sees it: who it is, and what it may
//! do (`rights_on`). Built by the auth middleware, read by the admin's gate,
//! Prisme and the developer's views.
use crate::auth::permissions::rang::Rang;
use crate::auth::permissions::{Groupe, Permission};
use crate::utils::pk::Pk;
use serde::{Deserialize, Serialize};

/// Authenticated user injected into request extensions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: Pk,
    pub username: String,
    pub rang: Rang,
    /// User groups (each group contains its permissions)
    pub groupes: Vec<Groupe>,
    /// An admin's own rights, set by the dirigeant on this account. Empty for
    /// every other rank: the dirigeant and the superuser have them all,
    /// lambda's come from this groups.
    #[serde(default)]
    pub droits: Vec<Permission>,
}

impl CurrentUser {
    /// The union of the groups' rights on one resource (logical OR). The
    /// groups alone: an admin's rights are on his account and a dirigeant's
    /// come from his rank, so only [`rights_on`](Self::rights_on) answers for a user.
    pub(crate) fn groups_union(&self, resource_key: &str) -> Option<Permission> {
        let mut union: Option<Permission> = None;
        for perm in self
            .groupes
            .iter()
            .flat_map(|g| &g.permissions)
            .filter(|p| p.resource_key == resource_key)
        {
            union
                .get_or_insert_with(|| Permission::zeroed(resource_key.to_string()))
                .merge_from(perm);
        }
        union
    }

    /// Whether the user may read `resource_key`, whatever his rank.
    #[must_use]
    pub fn can_access_resource(&self, resource_key: &str) -> bool {
        self.rights_on(resource_key)
            .crud
            .has(crate::auth::permissions::rights::Crud::Read)
    }

    /// Member of at least one group that opens the admin panel.
    #[must_use]
    pub fn is_staff(&self) -> bool {
        self.groupes.iter().any(|g| g.is_staff)
    }

    /// Checks if the user can access the admin panel.
    #[must_use]
    pub fn can_access_admin(&self) -> bool {
        self.rang > Rang::Lambda || self.is_staff()
    }
}
