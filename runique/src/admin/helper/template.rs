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
    /// Default template for the admin dashboard page (`admin/dashboard.html`), no dev override.
    pub fn dashboard() -> Self {
        Self {
            dev: None,
            runique: "admin/dashboard.html",
        }
    }
    /// Default template for the admin login page (`admin/login.html`), no dev override.
    pub fn login() -> Self {
        Self {
            dev: None,
            runique: "admin/login.html",
        }
    }
    /// Default template for the admin list view (`admin/list.html`), no dev override.
    pub fn list() -> Self {
        Self {
            dev: None,
            runique: "admin/list.html",
        }
    }
    /// Default template for the admin create view (`admin/create.html`), no dev override.
    pub fn create() -> Self {
        Self {
            dev: None,
            runique: "admin/create.html",
        }
    }
    /// Default template for the admin edit view (`admin/edit.html`), no dev override.
    pub fn edit() -> Self {
        Self {
            dev: None,
            runique: "admin/edit.html",
        }
    }
    /// Default template for the admin detail view (`admin/detail.html`), no dev override.
    pub fn detail() -> Self {
        Self {
            dev: None,
            runique: "admin/detail.html",
        }
    }
    /// Default template for the admin delete confirmation view (`admin/delete.html`), no dev override.
    pub fn delete() -> Self {
        Self {
            dev: None,
            runique: "admin/delete.html",
        }
    }
    /// Default base layout template (`admin_base.html`) that other admin templates extend.
    pub fn base() -> Self {
        Self {
            dev: None,
            runique: "admin_base.html",
        }
    }
    /// Default template for HTMX partial list responses (fragment only, no layout).
    pub fn htmx() -> Self {
        Self {
            dev: None,
            runique: "admin/list_partial.html",
        }
    }
    /// Default template for the admin bulk edit view (`admin/bulk_edit.html`), no dev override.
    pub fn bulk_edit() -> Self {
        Self {
            dev: None,
            runique: "admin/bulk_edit.html",
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
    pub bulk_edit: PathAdminTemplate,
}

impl AdminTemplate {
    /// Builds the default `AdminTemplate` configuration: every page points at
    /// its framework template with no dev override.
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
            bulk_edit: PathAdminTemplate::bulk_edit(),
        }
    }
    /// Overrides the dashboard template with a developer-provided path.
    #[must_use]
    pub fn with_dashboard(mut self, path: &str) -> Self {
        self.dashboard.dev = Some(path.to_string());
        self
    }
    /// Overrides the list view template with a developer-provided path.
    #[must_use]
    pub fn with_list(mut self, path: &str) -> Self {
        self.list.dev = Some(path.to_string());
        self
    }
    /// Overrides the create view template with a developer-provided path.
    #[must_use]
    pub fn with_create(mut self, path: &str) -> Self {
        self.create.dev = Some(path.to_string());
        self
    }
    /// Overrides the edit view template with a developer-provided path.
    #[must_use]
    pub fn with_edit(mut self, path: &str) -> Self {
        self.edit.dev = Some(path.to_string());
        self
    }
    /// Overrides the detail view template with a developer-provided path.
    #[must_use]
    pub fn with_detail(mut self, path: &str) -> Self {
        self.detail.dev = Some(path.to_string());
        self
    }
    /// Overrides the delete confirmation template with a developer-provided path.
    #[must_use]
    pub fn with_delete(mut self, path: &str) -> Self {
        self.delete.dev = Some(path.to_string());
        self
    }
    /// Overrides the login page template with a developer-provided path.
    #[must_use]
    pub fn with_login(mut self, path: &str) -> Self {
        self.login.dev = Some(path.to_string());
        self
    }
    /// Overrides the base layout template with a developer-provided path.
    #[must_use]
    pub fn with_base(mut self, path: &str) -> Self {
        self.base.dev = Some(path.to_string());
        self
    }
    /// Overrides the HTMX partial-list template with a developer-provided path.
    #[must_use]
    pub fn with_htmx(mut self, path: &str) -> Self {
        self.htmx.dev = Some(path.to_string());
        self
    }
    /// Overrides the bulk edit view template with a developer-provided path.
    #[must_use]
    pub fn with_bulk_edit(mut self, path: &str) -> Self {
        self.bulk_edit.dev = Some(path.to_string());
        self
    }
}

impl Default for AdminTemplate {
    fn default() -> Self {
        Self::new()
    }
}
