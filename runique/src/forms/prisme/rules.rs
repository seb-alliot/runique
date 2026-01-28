use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Contexte optionnel transmis via les extensions pour évaluer les règles.
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

/// Règles configurables pour Sentinel, à placer dans les extensions.
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

    /// Plusieurs rôles possibles, sans login imposé.
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

    /// Login requis + plusieurs rôles possibles.
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

/// Évalue les règles par rapport au contexte utilisateur. Retourne Ok si tout passe, sinon Response.
pub fn evaluate_rules(rules: &GuardRules, ctx: Option<&GuardContext>) -> Result<(), Box<Response>> {
    // Si aucune règle n'est définie, on laisse passer.
    if !rules.login_required && rules.roles.is_empty() {
        return Ok(());
    }

    let default_ctx = GuardContext::default();
    let ctx = ctx.unwrap_or(&default_ctx);

    if rules.login_required && !ctx.is_authenticated() {
        return Err(Box::new(
            (StatusCode::UNAUTHORIZED, "Authentication required").into_response(),
        ));
    }

    if !rules.roles.is_empty() {
        let has_any = rules.roles.iter().any(|role| ctx.has_role(role));
        if !has_any {
            return Err(Box::new(
                (StatusCode::FORBIDDEN, "Insufficient role").into_response(),
            ));
        }
    }

    Ok(())
}
