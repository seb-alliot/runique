//! Prisme guard rules: access control by role or authentication before body extraction.
use crate::utils::trad::t;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Optional context passed via extensions to evaluate rules.
#[derive(Debug, Clone, Default)]
pub struct GuardContext {
    pub user_id: Option<String>,
    pub roles: Vec<String>,
}

impl GuardContext {
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

/// Configurable rules for Sentinel, to be placed in extensions.
#[derive(Debug, Clone, Default)]
pub struct GuardRules {
    pub login_required: bool,
    pub roles: Vec<String>,
}

impl GuardRules {
    pub fn login_required() -> Self {
        Self {
            login_required: true,
            roles: Vec::new(),
        }
    }

    pub fn role(role: impl Into<String>) -> Self {
        Self {
            login_required: false,
            roles: vec![role.into()],
        }
    }

    /// Multiple possible roles, without mandatory login.
    pub fn roles<R, S>(roles: R) -> Self
    where
        R: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            login_required: false,
            roles: roles.into_iter().map(Into::into).collect(),
        }
    }

    pub fn login_and_role(role: impl Into<String>) -> Self {
        Self {
            login_required: true,
            roles: vec![role.into()],
        }
    }

    /// Login required + multiple possible roles.
    pub fn login_and_roles<R, S>(roles: R) -> Self
    where
        R: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            login_required: true,
            roles: roles.into_iter().map(Into::into).collect(),
        }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.roles.push(role.into());
        self
    }
}

/// Evaluates rules against the user context. Returns Ok if everything passes, otherwise Response.
pub fn evaluate_rules(rules: &GuardRules, ctx: Option<&GuardContext>) -> Result<(), Box<Response>> {
    // If no rule is defined, allow pass.
    if !rules.login_required && rules.roles.is_empty() {
        return Ok(());
    }

    let default_ctx = GuardContext::default();
    let ctx = ctx.unwrap_or(&default_ctx);

    if rules.login_required && !ctx.is_authenticated() {
        return Err(Box::new(
            (
                StatusCode::UNAUTHORIZED,
                t("forms.auth_required").into_owned(),
            )
                .into_response(),
        ));
    }

    if !rules.roles.is_empty() {
        let has_any = rules.roles.iter().any(|role| ctx.has_role(role));
        if !has_any {
            return Err(Box::new(
                (
                    StatusCode::FORBIDDEN,
                    t("forms.role_insufficient").into_owned(),
                )
                    .into_response(),
            ));
        }
    }

    Ok(())
}
