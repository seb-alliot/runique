//! Admin panel configuration: prefix, title, hot reload, and templates.
use std::sync::Arc;

use crate::admin::template::AdminTemplate;
use crate::auth::guard::LoginGuard;
use crate::middleware::security::RateLimiter;

/// Configuration for the admin panel: route prefix, branding, hot reload,
/// template overrides, and per-panel security settings
/// (rate limiting, login guard).
pub struct AdminConfig {
    /// Prefix for admin routes (default: "/admin")
    pub prefix: String,

    /// Title displayed in the admin interface (browser tab + sidebar).
    /// Empty when the developer has not called [`AdminConfig::site_title`]:
    /// templates then fall back to the translated `admin.base.title`, so a project
    /// that sets nothing follows the active language instead of a hardcoded string.
    pub site_title: String,

    /// The "view site" link of the admin's header. `None` gives `/`. Only set
    /// it when the public site is served by another project than the admin.
    pub view_site_url: Option<String>,

    /// Entirely enables or disables the `AdminPanel`
    pub enabled: bool,

    /// Admin template overrides (dashboard, login, list, etc.)
    pub templates: AdminTemplate,

    /// Number of entries per page in the list view (default: 10)
    pub page_size: u64,

    /// Values each filter group of the list shows before its arrows
    /// (default: 10)
    pub filter_size: u64,

    /// Longest search caractere maximum , default 150
    pub search_max_len: usize,

    /// Where the admin's emailed links lead and how long they work: set by
    /// the builder from `with_password_reset()`. `None` without it: the site
    /// serves no reset page, the admin sends no link.
    pub reset_link: Option<crate::auth::password::ResetLinkConfig>,

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
            site_title: self.site_title.clone(),
            view_site_url: self.view_site_url.clone(),
            enabled: self.enabled,
            templates: self.templates.clone(),
            page_size: self.page_size,
            filter_size: self.filter_size,
            search_max_len: self.search_max_len,
            reset_link: self.reset_link.clone(),
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
            .field("site_title", &self.site_title)
            .field("view_site_url", &self.view_site_url)
            .field("enabled", &self.enabled)
            .field("templates", &self.templates)
            .finish()
    }
}

impl AdminConfig {
    /// Builds the default admin configuration: prefix `/admin`, 10 rows per
    /// page, and framework-default templates.
    pub fn new() -> Self {
        Self {
            prefix: "/admin".to_string(),
            site_title: String::new(),
            view_site_url: None,
            enabled: true,
            templates: AdminTemplate::new(),
            page_size: 10,
            filter_size: crate::admin::builder::list::FILTER_PAGE_SIZE,
            search_max_len: 150,
            reset_link: None,
            reset_password_email_template: None,
            resource_order: Vec::new(),
            rate_limiter: None,
            login_guard: None,
        }
    }

    /// Sets the number of rows per page in list views (clamped to at least 1).
    pub fn page_size(mut self, size: u64) -> Self {
        self.page_size = size.max(1);
        self
    }

    /// Sets the values each filter group shows before its arrows (clamped
    /// to at least 1).
    pub fn filter_size(mut self, size: u64) -> Self {
        self.filter_size = size.max(1);
        self
    }

    /// Sets the longest search caractere
    /// clamped to at least 1, A longer is cut, never refused.
    pub fn search_max_len(mut self, len: usize) -> Self {
        self.search_max_len = len.max(1);
        self
    }

    /// Sets the URL prefix under which every admin route is mounted (default `/admin`).
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }

    /// Sets the title shown in the admin browser tab and sidebar, overriding
    /// the translated `admin.base.title` default.
    pub fn site_title(mut self, title: &str) -> Self {
        self.site_title = title.to_string();
        self
    }

    /// Changes the URL of the "view site" link, when the public site is served
    /// by another project than the admin.
    pub fn view_site_url(mut self, url: &str) -> Self {
        self.view_site_url = Some(url.to_string());
        self
    }

    /// Where the "view site" link leads: [`view_site_url`](Self::view_site_url), else `/`.
    pub fn view_site_href(&self) -> &str {
        self.view_site_url.as_deref().unwrap_or("/")
    }

    /// Disables the admin panel entirely.
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }

    /// Tera template for the password reset email from the admin.
    /// Context: `username`, `reset_url` and the `t_*` texts.
    pub fn reset_password_email_template(mut self, path: &str) -> Self {
        self.reset_password_email_template = Some(path.to_string());
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

#[cfg(test)]
mod tests {
    use super::AdminConfig;

    #[test]
    fn a_zero_is_raised_to_one() {
        assert_eq!(AdminConfig::new().page_size(0).page_size, 1);
        assert_eq!(AdminConfig::new().filter_size(0).filter_size, 1);
        assert_eq!(AdminConfig::new().search_max_len(0).search_max_len, 1);
    }
}
