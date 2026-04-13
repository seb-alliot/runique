//! Admin panel configuration: prefix, title, hot reload, auth, and templates.
use std::sync::Arc;

use crate::admin::helper::AdminTemplate;
use crate::auth::{guard::LoginGuard, session::AdminAuth};
use crate::middleware::security::RateLimiter;
use crate::utils::env::is_debug;

pub struct AdminConfig {
    /// Prefix for admin routes (default: "/admin")
    pub prefix: String,

    /// Enables the hot reload daemon in development
    pub hot_reload: bool,

    /// Title displayed in the admin interface
    pub site_title: String,

    /// Return URL to the main site (default: "/")
    pub site_url: String,

    /// Entirely enables or disables the `AdminPanel`
    pub enabled: bool,

    /// Admin login verification handler
    ///
    /// See `crate::auth::AdminAuth`.
    pub auth: Option<Arc<dyn AdminAuth>>,

    /// Admin template overrides (dashboard, login, list, etc.)
    pub templates: AdminTemplate,

    /// Number of entries per page in the list view (default: 10)
    pub page_size: u64,

    /// Base URL for password reset (default: None)
    /// The token will be added automatically: `{reset_password_url}/{token}`
    ///
    /// In production with a mailer, must be an absolute URL:
    /// `"https://mysite.com/reset-password"`
    ///
    /// If None, the link is displayed in the flash message (dev without mailer).
    pub reset_password_url: Option<String>,

    /// "User" resources: resource key → optional email template.
    /// Enable via `.user_resource("users")`.
    /// Upon creation, generates a random hashed password and sends a reset email.
    pub user_resources: std::collections::HashMap<String, Option<String>>,

    /// Tera template for the password reset email from the admin.
    /// Default: "admin/reset_password_email.html"
    /// Available context: `username`, `email`, `reset_url`
    pub reset_password_email_template: Option<String>,

    /// Display order of resources in the navigation (URL keys).
    /// Unlisted keys appear at the end in their insertion order.
    pub resource_order: Vec<String>,

    /// Rate limiter applied to the login route (optional).
    pub rate_limiter: Option<Arc<RateLimiter>>,

    /// Per-account brute-force protection on admin login (optional).
    pub login_guard: Option<Arc<LoginGuard>>,
}

impl Clone for AdminConfig {
    fn clone(&self) -> Self {
        Self {
            prefix: self.prefix.clone(),
            hot_reload: self.hot_reload,
            site_title: self.site_title.clone(),
            site_url: self.site_url.clone(),
            enabled: self.enabled,
            auth: self.auth.clone(),
            templates: self.templates.clone(),
            page_size: self.page_size,
            reset_password_url: self.reset_password_url.clone(),
            user_resources: self.user_resources.clone(),
            reset_password_email_template: self.reset_password_email_template.clone(),
            resource_order: self.resource_order.clone(),
            rate_limiter: self.rate_limiter.clone(),
            login_guard: self.login_guard.clone(),
        }
    }
}

impl std::fmt::Debug for AdminConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdminConfig")
            .field("prefix", &self.prefix)
            .field("hot_reload", &self.hot_reload)
            .field("site_title", &self.site_title)
            .field("site_url", &self.site_url)
            .field("enabled", &self.enabled)
            .field("auth", &self.auth.as_ref().map(|_| "<AdminAuth>"))
            .field("templates", &self.templates)
            .finish()
    }
}

impl AdminConfig {
    pub fn new() -> Self {
        Self {
            prefix: "/admin".to_string(),
            hot_reload: is_debug(),
            site_title: "Administration".to_string(),
            site_url: "/".to_string(),
            enabled: true,
            auth: None,
            templates: AdminTemplate::new(),
            page_size: 10,
            reset_password_url: None,
            user_resources: std::collections::HashMap::new(),
            reset_password_email_template: None,
            resource_order: Vec::new(),
            rate_limiter: None,
            login_guard: None,
        }
    }

    pub fn page_size(mut self, size: u64) -> Self {
        self.page_size = size.max(1);
        self
    }

    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.hot_reload = enabled;
        self
    }

    pub fn site_title(mut self, title: &str) -> Self {
        self.site_title = title.to_string();
        self
    }

    pub fn site_url(mut self, url: &str) -> Self {
        self.site_url = url.to_string();
        self
    }

    /// Base URL for password reset on the project side.
    /// The token will be added automatically: `{url}/{token}`
    ///
    /// In production (with a mailer), pass an absolute URL:
    /// ```rust,ignore
    /// .with_admin(|a| a.reset_password_url("https://mysite.com/reset-password"))
    /// ```
    ///
    /// To read from the environment in `main.rs`:
    /// ```rust,ignore
    /// let reset_url = std::env::var("RESET_PASSWORD_URL").ok();
    /// .with_admin(|a| {
    ///     let a = match &reset_url { Some(u) => a.reset_password_url(u), None => a };
    ///     a
    /// })
    /// ```
    pub fn reset_password_url(mut self, url: &str) -> Self {
        self.reset_password_url = Some(url.to_string());
        self
    }

    /// Attaches the admin authentication handler
    ///
    /// ```rust,ignore
    /// AdminConfig::new().auth(RuniqueAdminAuth::new())
    ///
    /// AdminConfig::new().auth(DefaultAdminAuth::<users::Entity>::new())
    /// ```
    pub fn auth<A: AdminAuth>(mut self, handler: A) -> Self {
        self.auth = Some(Arc::new(handler));
        self
    }

    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Declares a resource as a "user".
    /// Upon creation: random hashed password + reset email sent automatically.
    /// The form's email field must be named "email".
    ///
    /// ```rust,ignore
    /// AdminConfig::new().user_resource("users")
    /// ```
    pub fn user_resource(mut self, resource_key: &str) -> Self {
        self.user_resources.insert(resource_key.to_string(), None);
        self
    }

    /// Tera template for the password reset email from the admin.
    /// Context: `username`, `email`, `reset_url`
    pub fn reset_password_email_template(mut self, path: &str) -> Self {
        self.reset_password_email_template = Some(path.to_string());
        self
    }

    /// Like `user_resource` but with a custom email template.
    ///
    /// ```rust,ignore
    /// AdminConfig::new().user_resource_with_template("users", "emails/welcome.html")
    /// ```
    pub fn user_resource_with_template(mut self, resource_key: &str, email_template: &str) -> Self {
        self.user_resources
            .insert(resource_key.to_string(), Some(email_template.to_string()));
        self
    }

    /// Enables rate limiting on the admin login route.
    ///
    /// ```rust,ignore
    /// AdminConfig::new().with_rate_limiter(RateLimiter::new().max_requests(10).retry_after(60))
    /// ```
    pub fn with_rate_limiter(mut self, limiter: RateLimiter) -> Self {
        self.rate_limiter = Some(Arc::new(limiter));
        self
    }

    /// Enables per-account brute-force protection on the admin login.
    ///
    /// ```rust,ignore
    /// AdminConfig::new().with_login_guard(LoginGuard::new().max_attempts(5).lockout_secs(300))
    /// ```
    pub fn with_login_guard(mut self, guard: LoginGuard) -> Self {
        self.login_guard = Some(Arc::new(guard));
        self
    }

    /// Sets the display order of resources in the admin navigation.
    ///
    /// ```rust,ignore
    /// AdminConfig::new().resource_order(["users", "blog", "droits", "groupes"])
    /// ```
    /// Unlisted keys appear at the end in their insertion order.
    pub fn resource_order<I, S>(mut self, order: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.resource_order = order.into_iter().map(Into::into).collect();
        self
    }
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self::new()
    }
}
