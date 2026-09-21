//! Admin panel staging: configuration, routes, and admin state.

use std::sync::Arc;

use crate::admin::PrototypeAdminState;
use crate::admin::{AdminConfig, helper::AdminTemplate};
use crate::app::error_build::{BuildError, CheckError, CheckReport};
use crate::auth::{guard::LoginGuard, session::AdminAuth};
use crate::middleware::security::RateLimiter;
use axum::Router;

use crate::admin::AdminRoutes;

/// Admin panel configuration staged by the builder before the app is built.
/// Disabled by default — enabled once `.routes()` and `.auth()` are both
/// configured through `.with_admin(|a| ...)`.
pub struct AdminStaging {
    /// Resolved admin configuration: route prefix, auth handler, page size,
    /// rate limiting, login guard and templates.
    pub config: AdminConfig,
    /// Whether the admin panel is mounted at all.
    pub enabled: bool,
    /// Whether `/robots.txt` is generated automatically. Enabled by default.
    pub robots_txt: bool,
    /// `Sitemap:` directive added to the generated `/robots.txt`, if set.
    pub sitemap_url: Option<String>,
    /// CRUD router built from `.routes()`, mounted at the admin prefix.
    pub route_admin: Option<Router>,
    /// Extra routes registered via `.extra_routes()`, mounted within the
    /// admin's authentication boundary.
    pub extra_routes: Vec<(String, axum::routing::MethodRouter)>,
    /// Shared admin runtime state (registry, resources) injected via `.with_state()`.
    pub state: Option<Arc<PrototypeAdminState>>,

    /// Segment mounted in front of the admin (`.prefix()`), empty by default.
    /// Kept apart from `config.prefix` — which holds the admin's own path — so
    /// the two never overwrite each other, whatever the call order.
    pub(crate) mount_prefix: String,
}

/// `secret`, `/secret` and `/secret/` all yield `/secret`; empty stays empty.
fn normalize_segment(raw: &str) -> String {
    let trimmed = raw.trim().trim_matches('/');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("/{trimmed}")
    }
}

impl AdminStaging {
    /// Creates a disabled `AdminStaging` with default configuration.
    pub fn new() -> Self {
        Self {
            config: AdminConfig::new(),
            enabled: false,
            robots_txt: true,
            sitemap_url: None,
            route_admin: None,
            extra_routes: Vec::new(),
            state: None,
            mount_prefix: String::new(),
        }
    }

    /// Disables the automatic generation of `/robots.txt` (enabled by default).
    pub fn no_robots_txt(mut self) -> Self {
        self.robots_txt = false;
        self
    }

    /// Adds a `Sitemap:` directive to the generated `/robots.txt`.
    ///
    /// ```rust,ignore
    /// .with_admin(|a| a.sitemap("https://mysite.com/sitemap.xml"))
    /// ```
    pub fn sitemap(mut self, url: &str) -> Self {
        self.sitemap_url = Some(url.to_string());
        self
    }

    /// Mounts the generated CRUD routes and records the admin's own path.
    ///
    /// Independent from [`AdminStaging::prefix`]: this sets *where the admin
    /// lives*, `prefix()` sets *what is put in front of it*.
    pub fn routes(mut self, admin_routes: AdminRoutes) -> Self {
        self.config.prefix = admin_routes.path;
        self.route_admin = Some(admin_routes.router);
        self
    }

    /// Registers additional routes within the admin middleware boundary.
    ///
    /// Paths are relative to the admin prefix — the framework prepends it automatically.
    /// These routes inherit admin authentication, `AdminState` and `PrototypeAdminState`.
    ///
    /// ```rust,ignore
    /// // url.rs
    /// pub fn admin_extra_routes() -> Vec<(&'static str, runique::axum::routing::MethodRouter)> {
    ///     vec![
    ///         ("/commandes/{numero}/detail", view!{ admin_commande_detail }),
    ///     ]
    /// }
    ///
    /// // main.rs
    /// .with_admin(|a| a.extra_routes(url::admin_extra_routes()))
    /// ```
    pub fn extra_routes(mut self, routes: Vec<(&str, axum::routing::MethodRouter)>) -> Self {
        for (path, method_router) in routes {
            let path = format!("/{}", path.trim_start_matches('/'));
            self.extra_routes.push((path, method_router));
        }
        self
    }

    /// Injects the shared admin runtime state (registry and resources).
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

    /// Enables or disables hot-reloading of admin templates.
    pub fn hot_reload(mut self, enabled: bool) -> Self {
        self.config = self.config.hot_reload(enabled);
        self
    }

    /// Sets the site title shown in the admin interface.
    pub fn site_title(mut self, title: &str) -> Self {
        self.config = self.config.site_title(title);
        self
    }

    /// Sets the public site URL used to build absolute links in the admin.
    pub fn site_url(mut self, url: &str) -> Self {
        self.config = self.config.site_url(url);
        self
    }

    /// Prepends a segment in front of the whole admin, without renaming it.
    ///
    /// The admin's own path comes from `.routes(admins::routes("/site-admin"))`;
    /// this prefix is mounted in front of it, so `.prefix("secret")` serves it
    /// under `/secret/site-admin/…`. The two are independent: calling them in
    /// either order gives the same result.
    ///
    /// Leading and trailing slashes are optional — `secret`, `/secret` and
    /// `/secret/` are equivalent.
    pub fn prefix(mut self, prefix: &str) -> Self {
        self.mount_prefix = normalize_segment(prefix);
        self
    }

    /// Public base URL of the admin: mount prefix + admin path.
    ///
    /// Single source for every generated link (`admin_prefix` in templates,
    /// `scope_base`, redirects) — the router is mounted so that it answers
    /// exactly here.
    pub(crate) fn public_prefix(&self) -> String {
        format!(
            "{}{}",
            self.mount_prefix,
            self.config.prefix.trim_end_matches('/')
        )
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

    /// Disables the admin panel entirely.
    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self.config = self.config.disable();
        self
    }

    /// Enables the admin panel.
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

    /// Checks the staged configuration when the admin is enabled: the route
    /// prefix must be non-empty and an authentication handler must be set.
    /// A no-op returning `Ok(())` when the admin is disabled.
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

    /// Returns `true` if the admin is disabled, or enabled with a non-empty
    /// route prefix configured.
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
