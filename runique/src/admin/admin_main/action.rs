//! Admin actions reified as values (Command pattern).
//!
//! Each action carries its own authorization rule, so the access policy lives
//! in one place instead of being duplicated across the four entry points
//! (`admin_get`/`admin_post`/`admin_get_id`/`admin_post_id`). Parsing also
//! rejects actions that are invalid for the HTTP method, mirroring the
//! `not_found` arms of the previous string matches.

use super::ResourcePerms;

/// Outcome of authorizing an action against the resolved permissions.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Access {
    Granted,
    /// Insufficient rights to even see the resource → redirect to dashboard.
    DeniedDashboard,
    /// Resource visible but this operation is forbidden → redirect to its list.
    DeniedResource,
}

/// Actions on a collection URL: `/admin/{resource}/{action}`.
pub(super) enum CollectionAction {
    List,
    Create,
    Bulk,
}

/// Actions on a member URL: `/admin/{resource}/{id}/{action}`.
pub(super) enum MemberAction {
    Detail,
    Edit,
    Delete,
    ResetPassword,
}

impl CollectionAction {
    /// Valid GET actions on a collection (`None` → 404).
    pub(super) fn parse_get(action: &str) -> Option<Self> {
        match action {
            "list" => Some(Self::List),
            "create" => Some(Self::Create),
            "bulk" => Some(Self::Bulk),
            _ => None,
        }
    }

    /// Valid POST actions on a collection (`None` → 404). `list` is GET-only.
    pub(super) fn parse_post(action: &str) -> Option<Self> {
        match action {
            "create" => Some(Self::Create),
            "bulk" => Some(Self::Bulk),
            _ => None,
        }
    }

    pub(super) fn authorize_get(&self, perms: &ResourcePerms) -> Access {
        match self {
            Self::List => {
                if perms.can_read {
                    Access::Granted
                } else {
                    Access::DeniedDashboard
                }
            }
            Self::Create => {
                if !perms.can_read {
                    Access::DeniedDashboard
                } else if perms.can_create {
                    Access::Granted
                } else {
                    Access::DeniedResource
                }
            }
            Self::Bulk => {
                if perms.can_update {
                    Access::Granted
                } else {
                    Access::DeniedResource
                }
            }
        }
    }

    /// `bulk_action` is the `bulk_action` form field ("delete" selects the
    /// delete right, anything else the update right).
    ///
    /// The `can_create` requirement on the collection POST is the pre-existing
    /// behavior, kept verbatim so this refactor changes nothing.
    pub(super) fn authorize_post(&self, perms: &ResourcePerms, bulk_action: &str) -> Access {
        if !perms.can_create {
            return Access::DeniedResource;
        }
        match self {
            Self::Create => Access::Granted,
            Self::Bulk => {
                let can_bulk = if bulk_action == "delete" {
                    perms.can_delete
                } else {
                    perms.can_update
                };
                if can_bulk {
                    Access::Granted
                } else {
                    Access::DeniedResource
                }
            }
            // `list` is rejected at parse time; unreachable in practice.
            Self::List => Access::DeniedResource,
        }
    }
}

impl MemberAction {
    /// Valid GET actions on a member (`None` → 404).
    pub(super) fn parse_get(action: &str) -> Option<Self> {
        match action {
            "detail" => Some(Self::Detail),
            "edit" => Some(Self::Edit),
            "delete" => Some(Self::Delete),
            _ => None,
        }
    }

    /// Valid POST actions on a member (`None` → 404).
    pub(super) fn parse_post(action: &str) -> Option<Self> {
        match action {
            "edit" => Some(Self::Edit),
            "delete" => Some(Self::Delete),
            "reset-password" => Some(Self::ResetPassword),
            _ => None,
        }
    }

    /// Per-action rule for a member operation. The `can_read` dashboard gate is
    /// applied separately by the GET entry point (so `owns_record` keeps being
    /// resolved only after that gate, as before). `reset-password` is a
    /// sensitive write → same rule as edit.
    pub(super) fn authorize(&self, perms: &ResourcePerms, owns: bool) -> Access {
        match self {
            Self::Detail => Access::Granted,
            Self::Edit | Self::ResetPassword => grant_if(perms.can_edit(owns)),
            Self::Delete => grant_if(perms.can_remove(owns)),
        }
    }
}

fn grant_if(allowed: bool) -> Access {
    if allowed {
        Access::Granted
    } else {
        Access::DeniedResource
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::permissions::{Groupe, Permission};
    use crate::auth::session::CurrentUser;

    /// Builds a `ResourcePerms` from the six flags in declaration order:
    /// create, read, update, delete, update_own, delete_own.
    fn perms(c: bool, r: bool, u: bool, d: bool, uo: bool, dorm: bool) -> ResourcePerms {
        ResourcePerms {
            can_create: c,
            can_read: r,
            can_update: u,
            can_delete: d,
            can_update_own: uo,
            can_delete_own: dorm,
        }
    }

    const NONE: ResourcePerms = ResourcePerms {
        can_create: false,
        can_read: false,
        can_update: false,
        can_delete: false,
        can_update_own: false,
        can_delete_own: false,
    };

    // ── Parsing rejects actions invalid for the method ───────────────

    #[test]
    fn collection_parse_rejects_cross_method() {
        assert!(CollectionAction::parse_get("list").is_some());
        assert!(CollectionAction::parse_post("list").is_none()); // list is GET-only
        assert!(CollectionAction::parse_get("nope").is_none());
        assert!(CollectionAction::parse_post("create").is_some());
    }

    #[test]
    fn member_parse_rejects_cross_method() {
        assert!(MemberAction::parse_get("detail").is_some());
        assert!(MemberAction::parse_post("detail").is_none()); // detail is GET-only
        assert!(MemberAction::parse_get("reset-password").is_none()); // POST-only
        assert!(MemberAction::parse_post("reset-password").is_some());
    }

    // ── Collection GET ───────────────────────────────────────────────

    #[test]
    fn list_requires_read() {
        assert_eq!(
            CollectionAction::List.authorize_get(&perms(false, true, false, false, false, false)),
            Access::Granted
        );
        assert_eq!(
            CollectionAction::List.authorize_get(&NONE),
            Access::DeniedDashboard
        );
    }

    #[test]
    fn create_get_is_two_stage() {
        // no read at all → dashboard
        assert_eq!(
            CollectionAction::Create.authorize_get(&NONE),
            Access::DeniedDashboard
        );
        // read but no create → resource list
        assert_eq!(
            CollectionAction::Create.authorize_get(&perms(false, true, false, false, false, false)),
            Access::DeniedResource
        );
        // read + create → granted
        assert_eq!(
            CollectionAction::Create.authorize_get(&perms(true, true, false, false, false, false)),
            Access::Granted
        );
    }

    #[test]
    fn bulk_get_requires_update_no_dashboard() {
        assert_eq!(
            CollectionAction::Bulk.authorize_get(&perms(false, false, true, false, false, false)),
            Access::Granted
        );
        // no update → resource (never dashboard)
        assert_eq!(
            CollectionAction::Bulk.authorize_get(&NONE),
            Access::DeniedResource
        );
    }

    // ── Collection POST (preserves the can_create upfront gate) ──────

    #[test]
    fn create_post_requires_create() {
        assert_eq!(
            CollectionAction::Create
                .authorize_post(&perms(true, false, false, false, false, false), ""),
            Access::Granted
        );
        assert_eq!(
            CollectionAction::Create.authorize_post(&NONE, ""),
            Access::DeniedResource
        );
    }

    #[test]
    fn bulk_post_needs_create_plus_operation_right() {
        // create + update, non-delete action → granted
        assert_eq!(
            CollectionAction::Bulk
                .authorize_post(&perms(true, false, true, false, false, false), "activate"),
            Access::Granted
        );
        // create + delete-right, delete action → granted
        assert_eq!(
            CollectionAction::Bulk
                .authorize_post(&perms(true, false, false, true, false, false), "delete"),
            Access::Granted
        );
        // has update but NOT create → denied (preserved upfront gate)
        assert_eq!(
            CollectionAction::Bulk
                .authorize_post(&perms(false, false, true, false, false, false), "activate"),
            Access::DeniedResource
        );
        // create but delete action without delete right → denied
        assert_eq!(
            CollectionAction::Bulk
                .authorize_post(&perms(true, false, true, false, false, false), "delete"),
            Access::DeniedResource
        );
    }

    // ── Member actions ───────────────────────────────────────────────

    #[test]
    fn detail_always_granted_after_gate() {
        assert_eq!(
            MemberAction::Detail.authorize(&NONE, false),
            Access::Granted
        );
    }

    #[test]
    fn edit_global_vs_own() {
        // global update → granted regardless of ownership
        assert_eq!(
            MemberAction::Edit.authorize(&perms(false, false, true, false, false, false), false),
            Access::Granted
        );
        // only update_own + owns the record → granted
        assert_eq!(
            MemberAction::Edit.authorize(&perms(false, false, false, false, true, false), true),
            Access::Granted
        );
        // update_own but does NOT own → denied
        assert_eq!(
            MemberAction::Edit.authorize(&perms(false, false, false, false, true, false), false),
            Access::DeniedResource
        );
        // nothing → denied
        assert_eq!(
            MemberAction::Edit.authorize(&NONE, true),
            Access::DeniedResource
        );
    }

    #[test]
    fn delete_global_vs_own() {
        assert_eq!(
            MemberAction::Delete.authorize(&perms(false, false, false, true, false, false), false),
            Access::Granted
        );
        assert_eq!(
            MemberAction::Delete.authorize(&perms(false, false, false, false, false, true), true),
            Access::Granted
        );
        assert_eq!(
            MemberAction::Delete.authorize(&perms(false, false, false, false, false, true), false),
            Access::DeniedResource
        );
    }

    #[test]
    fn reset_password_follows_edit_rule() {
        // same rule as edit: global update grants it
        assert_eq!(
            MemberAction::ResetPassword
                .authorize(&perms(false, false, true, false, false, false), false),
            Access::Granted
        );
        // update_own + owns
        assert_eq!(
            MemberAction::ResetPassword
                .authorize(&perms(false, false, false, false, true, false), true),
            Access::Granted
        );
        // no update right → denied (delete right must NOT grant a reset)
        assert_eq!(
            MemberAction::ResetPassword
                .authorize(&perms(false, false, false, true, false, false), true),
            Access::DeniedResource
        );
    }

    // ── ResourcePerms::resolve ───────────────────────────────────────

    fn user(is_superuser: bool, groupes: Vec<Groupe>) -> CurrentUser {
        CurrentUser {
            id: 1,
            username: "tester".to_string(),
            is_staff: true,
            is_superuser,
            groupes,
        }
    }

    #[test]
    fn resolve_superuser_grants_everything() {
        let p = ResourcePerms::resolve(&user(true, vec![]), "anything");
        assert!(
            p.can_create
                && p.can_read
                && p.can_update
                && p.can_delete
                && p.can_update_own
                && p.can_delete_own
        );
    }

    #[test]
    fn resolve_reads_matching_permission() {
        let mut perm = Permission::zeroed("blog".to_string());
        perm.can_read = true;
        perm.can_update = true;
        let groupe = Groupe {
            id: 1,
            nom: "editors".to_string(),
            permissions: vec![perm],
        };
        let p = ResourcePerms::resolve(&user(false, vec![groupe]), "blog");
        assert!(p.can_read && p.can_update);
        assert!(!p.can_create && !p.can_delete);
    }

    #[test]
    fn resolve_unknown_resource_is_all_false() {
        let p = ResourcePerms::resolve(&user(false, vec![]), "ghost");
        assert_eq!(
            (
                p.can_create,
                p.can_read,
                p.can_update,
                p.can_delete,
                p.can_update_own,
                p.can_delete_own
            ),
            (false, false, false, false, false, false)
        );
    }
}
