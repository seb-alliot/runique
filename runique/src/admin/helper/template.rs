//! Admin template management with optional developer override.

/// Admin template path — with fallback to Runique default.
///
/// `runique` : Tera key of the framework template (immutable)
/// `dev`     : optional dev path — replaces `runique` if defined
#[derive(Debug, Clone)]
pub struct PathAdminTemplate {
    pub dev: Option<String>,
    pub runique: &'static str,
}

impl PathAdminTemplate {
    /// Returns the resolved path: dev has priority, otherwise Runique default.
    #[must_use]
    pub fn resolve(&self) -> &str {
        self.dev.as_deref().unwrap_or(self.runique)
    }
    pub fn dashboard() -> Self {
        Self {
            dev: None,
            runique: "admin/dashboard",
        }
    }
    pub fn login() -> Self {
        Self {
            dev: None,
            runique: "admin/login",
        }
    }
    pub fn list() -> Self {
        Self {
            dev: None,
            runique: "admin/list",
        }
    }
    pub fn create() -> Self {
        Self {
            dev: None,
            runique: "admin/create",
        }
    }
    pub fn edit() -> Self {
        Self {
            dev: None,
            runique: "admin/edit",
        }
    }
    pub fn detail() -> Self {
        Self {
            dev: None,
            runique: "admin/detail",
        }
    }
    pub fn delete() -> Self {
        Self {
            dev: None,
            runique: "admin/delete",
        }
    }
    pub fn base() -> Self {
        Self {
            dev: None,
            runique: "admin_base",
        }
    }
    pub fn htmx() -> Self {
        Self {
            dev: None,
            runique: "admin/list_partial",
        }
    }
}

/// Global configuration for admin templates.
///
/// Resolution hierarchy (decreasing priority):
/// 1. `admin!{ template_list: "..." }` — override per resource
/// 2. `AdminTemplate.list.dev`         — global dev override
/// 3. `AdminTemplate.list.runique`     — framework default
///
/// ## Example
/// ```rust,ignore
/// .with_admin(|a| a
///     .templates(|t| t
///         .with_list("my_theme/list")
///         .with_dashboard("my_theme/dashboard")
///     )
/// )
/// ```
#[derive(Debug, Clone)]
pub struct AdminTemplate {
    pub dashboard: PathAdminTemplate,
    pub list: PathAdminTemplate,
    pub create: PathAdminTemplate,
    pub edit: PathAdminTemplate,
    pub detail: PathAdminTemplate,
    pub delete: PathAdminTemplate,
    pub login: PathAdminTemplate,
    pub base: PathAdminTemplate,
    /// Template for HTMX partial responses (fragment only, no layout).
    pub htmx: PathAdminTemplate,
}

impl AdminTemplate {
    pub fn new() -> Self {
        Self {
            dashboard: PathAdminTemplate::dashboard(),
            list: PathAdminTemplate::list(),
            create: PathAdminTemplate::create(),
            edit: PathAdminTemplate::edit(),
            detail: PathAdminTemplate::detail(),
            delete: PathAdminTemplate::delete(),
            login: PathAdminTemplate::login(),
            base: PathAdminTemplate::base(),
            htmx: PathAdminTemplate::htmx(),
        }
    }
    #[must_use]
    pub fn with_dashboard(mut self, path: &str) -> Self {
        self.dashboard.dev = Some(path.to_string());
        self
    }
    #[must_use]
    pub fn with_list(mut self, path: &str) -> Self {
        self.list.dev = Some(path.to_string());
        self
    }
    #[must_use]
    pub fn with_create(mut self, path: &str) -> Self {
        self.create.dev = Some(path.to_string());
        self
    }
    #[must_use]
    pub fn with_edit(mut self, path: &str) -> Self {
        self.edit.dev = Some(path.to_string());
        self
    }
    #[must_use]
    pub fn with_detail(mut self, path: &str) -> Self {
        self.detail.dev = Some(path.to_string());
        self
    }
    #[must_use]
    pub fn with_delete(mut self, path: &str) -> Self {
        self.delete.dev = Some(path.to_string());
        self
    }
    #[must_use]
    pub fn with_login(mut self, path: &str) -> Self {
        self.login.dev = Some(path.to_string());
        self
    }
    #[must_use]
    pub fn with_base(mut self, path: &str) -> Self {
        self.base.dev = Some(path.to_string());
        self
    }
    #[must_use]
    pub fn with_htmx(mut self, path: &str) -> Self {
        self.htmx.dev = Some(path.to_string());
        self
    }
}

impl Default for AdminTemplate {
    fn default() -> Self {
        Self::new()
    }
}
