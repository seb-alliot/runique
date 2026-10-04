//! Prisme guard rules: access control before body extraction, checked against
//! the signed-in account as `auth_middleware` read it from the database.
use crate::auth::session::CurrentUser;
use crate::utils::trad::t;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Configurable rules for Sentinel, to be placed in extensions.
///
/// Every rule requires a signed-in account. A superuser passes the staff and
/// role checks, as everywhere else in Runique.
#[derive(Debug, Clone, Default)]
pub struct GuardRules {
    pub login_required: bool,
    pub staff: bool,
    pub superuser: bool,
    /// Group names (`eihwaz_groupes.nom`): belonging to any one of them is enough.
    pub roles: Vec<String>,
}

impl GuardRules {
    /// Rule requiring only that the request be authenticated.
    pub fn login_required() -> Self {
        Self {
            login_required: true,
            ..Self::default()
        }
    }

    /// Rule requiring a staff (or superuser) account.
    pub fn staff() -> Self {
        Self {
            login_required: true,
            staff: true,
            ..Self::default()
        }
    }

    /// Rule requiring a superuser account.
    pub fn superuser() -> Self {
        Self {
            login_required: true,
            superuser: true,
            ..Self::default()
        }
    }

    /// Rule requiring membership of any one of the groups named in `roles`
    /// (`roles(["editeur"])` for a single one).
    pub fn roles<R, S>(roles: R) -> Self
    where
        R: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            login_required: true,
            roles: roles.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }

    fn is_empty(&self) -> bool {
        !self.login_required && !self.staff && !self.superuser && self.roles.is_empty()
    }
}

/// Evaluates `rules` against the signed-in account. `groupes` holds the
/// account's group names; [`sentinel`](super::sentinel) loads them only when
/// `rules.roles` asks for them.
pub fn evaluate_rules(
    rules: &GuardRules,
    user: Option<&CurrentUser>,
    groupes: &[String],
) -> Result<(), Box<Response>> {
    if rules.is_empty() {
        return Ok(());
    }
    let Some(user) = user else {
        return Err(refusal(StatusCode::UNAUTHORIZED, "forms.auth_required"));
    };
    let allowed = (!rules.staff || user.is_staff || user.is_superuser)
        && (!rules.superuser || user.is_superuser)
        && (rules.roles.is_empty()
            || user.is_superuser
            || rules.roles.iter().any(|role| groupes.contains(role)));
    if allowed {
        Ok(())
    } else {
        Err(refusal(StatusCode::FORBIDDEN, "forms.role_insufficient"))
    }
}

fn refusal(status: StatusCode, key: &str) -> Box<Response> {
    Box::new((status, t(key).into_owned()).into_response())
}
