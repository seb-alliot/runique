//! Admin panel staging: configuration, routes, and admin state.

use std::sync::Arc;

use crate::admin::PrototypeAdminState;
use crate::admin::{AdminConfig, helper::AdminTemplate};
use crate::app::error_build::{BuildError, CheckError, CheckReport};
use crate::auth::{guard::LoginGuard, session::AdminAuth};
use crate::middleware::security::RateLimiter;
use axum::Router;

pub struct AdminStaging {
    pub config: AdminConfig,
    pub enabled: bool,
    pub robots_txt: bool,
    pub route_admin: Option<Router>,
    pub state: Option<Arc<PrototypeAdminState>>,
}

impl AdminStaging {
    pub fn new() -> Self {
        Self {
            config: AdminConfig::new(),
            enabled: false,
            robots_txt: true,
            route_admin: None,
            state: None,
        }
    }

    /// Disables the automatic generation of `/robots.txt` (enabled by default).
    pub fn no_robots_txt(mut self) -> Self {
        self.robots_txt = false;
        self
    }

    pub fn routes(mut self, router: Router) -> Self {
        self.route_admin = Some(router);
        self
    }

    pub fn with_state(mut self, state: Arc<PrototypeAdminState>) -> Self {
        self.state = Some(state);
        self
    }

    /// Sets the resource display order in the admin navigation.
    ///
    /// ```rust,ignore
    /// .with_admin(|a| a
    ///     .resource_order(["users", "blog", "permissions", "groups"])
    /// )
    /// ```
    /// Unlisted keys appear at the end in their insertion order.
    pub fn resource_order<I, S>(mut self, order: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.config = self.config.resource_order(order);
        self
    }

    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.config = self.config.hot_reload(enabled);
        self
    }

    pub fn site_title(mut self, title: &str) -> Self {
        self.config = self.config.site_title(title);
        self
    }

    pub fn site_url(mut self, url: &str) -> Self {
        self.config = self.config.site_url(url);
        self
    }

    /// Sets the prefix for admin routes (default: `/admin`).
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.config = self.config.prefix(prefix);
        self
    }

    /// Sets the number of entries per page in the list view (default: 10).
    pub fn page_size(mut self, size: u64) -> Self {
        self.config = self.config.page_size(size);
        self
    }

    /// Connects the admin authentication handler.
    ///
    /// ## With built-in User (zero config):
    /// ```rust,ignore
    /// use runique::auth::RuniqueAdminAuth;
    ///
    /// .with_admin(|a| a
    ///     .site_title("My Admin")
    ///     .auth(RuniqueAdminAuth::new())
    /// )
    /// ```
    ///
    /// ## With a custom model:
    /// ```rust,ignore
    /// use runique::auth::{DefaultAdminAuth, UserEntity};
    ///
    /// impl UserEntity for users::Entity { ... }
    ///
    /// .with_admin(|a| a.auth(DefaultAdminAuth::<users::Entity>::new()))
    /// ```
    pub fn auth<A: AdminAuth>(mut self, handler: A) -> Self {
        self.config = self.config.auth(handler);
        self
    }

    /// Enables rate limiting on the admin login route.
    ///
    /// ```rust,ignore
    /// .with_admin(|a| a.with_rate_limiter(RateLimiter::new().max_requests(10).retry_after(60)))
    /// ```
    pub fn with_rate_limiter(mut self, limiter: RateLimiter) -> Self {
        self.config = self.config.with_rate_limiter(limiter);
        self
    }

    /// Enables per-account brute-force protection on the admin login.
    ///
    /// ```rust,ignore
    /// .with_admin(|a| a.with_login_guard(LoginGuard::new().max_attempts(5).lockout_secs(300)))
    /// ```
    pub fn with_login_guard(mut self, guard: LoginGuard) -> Self {
        self.config = self.config.with_login_guard(guard);
        self
    }

    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self.config = self.config.disable();
        self
    }

    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Overrides admin interface templates.
    ///
    /// ```rust,ignore
    /// .with_admin(|a| a
    ///     .templates(|t| t
    ///         .with_list("my_theme/list.html")
    ///         .with_dashboard("my_theme/dashboard.html")
    ///     )
    /// )
    /// ```
    pub fn templates<F: FnOnce(AdminTemplate) -> AdminTemplate>(mut self, f: F) -> Self {
        let current = std::mem::take(&mut self.config.templates);
        self.config.templates = f(current);
        self
    }

    pub fn validate(&self) -> Result<(), BuildError> {
        if !self.enabled {
            return Ok(());
        }

        let mut report = CheckReport::new();

        if self.config.prefix.is_empty() {
            report.add(
                CheckError::new("AdminPanel", "The admin route prefix cannot be empty")
                    .with_suggestion("Use .prefix(\"/admin\") or leave the default value"),
            );
        }

        if self.config.auth.is_none() {
            report.add(
                CheckError::new("AdminPanel", "No authentication handler configured")
                    .with_suggestion(
                        "Add .auth(RuniqueAdminAuth::new()) to use the built-in User, \
                    or implement UserEntity on your own model",
                    ),
            );
        }

        if report.has_errors() {
            return Err(BuildError::check(report));
        }

        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        if !self.enabled {
            return true;
        }
        !self.config.prefix.is_empty()
    }
}

impl Default for AdminStaging {
    fn default() -> Self {
        Self::new()
    }
}
