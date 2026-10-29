//! Admin panel staging: configuration, routes, and the registered models.

use crate::admin::builder::site::AdminSite;
use crate::admin::router::AdminView;
use crate::admin::{AdminConfig, template::AdminTemplate};
use crate::app::error_build::{BuildError, CheckError, CheckReport};
use crate::auth::guard::LoginGuard;
use crate::middleware::security::RateLimiter;

/// An admin route declaration retained from `urlpatterns!` until build.
pub(crate) struct DeclaredAdminRoute {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) model_key: String,
    pub(crate) handler: axum::routing::MethodRouter,
    pub(crate) overrides: AdminView,
}

fn paths_overlap(left: &str, right: &str) -> bool {
    let left: Vec<_> = left
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    let right: Vec<_> = right
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    for (a, b) in left.iter().zip(right.iter()) {
        if a.starts_with("{*") || b.starts_with("{*") {
            return true;
        }
        if a != b
            && !(a.starts_with('{') && a.ends_with('}'))
            && !(b.starts_with('{') && b.ends_with('}'))
        {
            return false;
        }
    }
    left.len() == right.len()
}

/// Admin panel configuration staged by the builder before the app is built.
/// Disabled by default — enabled and configured by `.with_admin(|a| ...)`.
/// Accounts are those of `eihwaz_users`, let in when active and staff or
/// superuser.
pub struct AdminStaging {
    /// Resolved admin configuration: route prefix, page size,
    /// rate limiting, login guard and templates.
    pub config: AdminConfig,
    /// Whether the admin panel is mounted at all.
    pub enabled: bool,
    /// Whether `/robots.txt` is generated automatically. Enabled by default.
    pub robots_txt: bool,
    /// `Sitemap:` directive added to the generated `/robots.txt`, if set.
    pub sitemap_url: Option<String>,
    /// Declarative admin views from `urlpatterns!`.
    pub(crate) declared_routes: Vec<DeclaredAdminRoute>,
    /// Paths declared as public routes in the same `urlpatterns!` set.
    pub(crate) public_route_paths: Vec<String>,

    /// Segment mounted in front of the admin (`.prefix()`), empty by default.
    /// Kept apart from `config.prefix` — which holds the admin's own path — so
    /// the two never overwrite each other, whatever the call order.
    pub(crate) mount_prefix: String,

    /// Models registered in the rebuilt admin, in the order they were added;
    /// checked at startup by `validate()`.
    pub(crate) site: AdminSite,
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
            declared_routes: Vec::new(),
            public_route_paths: Vec::new(),
            mount_prefix: String::new(),
            site: AdminSite::default(),
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

    /// Records the admin's own path (`/site-admin`).
    ///
    /// Independent from [`AdminStaging::prefix`]: this sets *where the admin
    /// lives*, `prefix()` sets *what is put in front of it*.
    pub fn path(mut self, path: &str) -> Self {
        self.config.prefix = path.to_string();
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
    pub fn site(mut self, f: impl FnOnce(AdminSite) -> AdminSite) -> Self {
        self.site = f(self.site);
        self
    }
    /// Sets the admin's "view site" link, when the public site is served by
    /// another project than the admin. By default it follows the app's public
    /// URL (`.with_public_url()`), else `/`.
    pub fn view_site_url(mut self, url: &str) -> Self {
        self.config = self.config.view_site_url(url);
        self
    }

    /// Prepends a segment in front of the whole admin, without renaming it.
    ///
    /// The admin's own path comes from [`.path("/site-admin")`](Self::path);
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

    /// Sets the longest caractere in search  sql.
    pub fn search_max_len(mut self, len: usize) -> Self {
        self.config = self.config.search_max_len(len);
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
            if !self.declared_routes.is_empty() {
                return Err(BuildError::validation(
                    "admin routes were declared in `urlpatterns!`, but the admin is disabled",
                ));
            }
            return Ok(());
        }

        let mut report = CheckReport::new();

        if self.config.prefix.is_empty() {
            report.add(
                CheckError::new(
                    "AdminPanel",
                    crate::utils::trad::t("build.check.admin_prefix_empty"),
                )
                .with_suggestion(crate::utils::trad::t("build.check.admin_prefix_empty_hint")),
            );
        }

        for (index, route) in self.declared_routes.iter().enumerate() {
            if self.site.model(&route.model_key).is_none() {
                report.add(CheckError::new(
                    "AdminPanel",
                    format!(
                        "admin route `{}` references unregistered model `{}`",
                        route.path, route.model_key
                    ),
                ));
            }
            if !route.path.starts_with('/')
                || (route.path != "/" && route.path.ends_with('/'))
                || matches!(
                    route.overrides,
                    AdminView::Detail
                        // The row's link fills `{id}` and nothing else.
                        if route
                            .path
                            .split('/')
                            .filter(|part| part.starts_with('{'))
                            .ne(["{id}"])
                )
            {
                report.add(CheckError::new(
                    "AdminPanel",
                    format!(
                        "admin route `{}` must be an absolute path{}",
                        route.path,
                        if route.overrides == AdminView::Detail {
                            " with exactly one `{id}` segment, and no other parameter, to override Detail"
                        } else {
                            ""
                        }
                    ),
                ));
            }
            if self
                .public_route_paths
                .iter()
                .any(|path| paths_overlap(path, &route.path))
            {
                report.add(CheckError::new(
                    "AdminPanel",
                    format!(
                        "admin route `{}` overlaps a public route in `urlpatterns!`",
                        route.path
                    ),
                ));
            }
            let admin_prefix = self.public_prefix();
            if route.path == admin_prefix || route.path.starts_with(&format!("{admin_prefix}/")) {
                report.add(CheckError::new(
                    "AdminPanel",
                    format!(
                        "admin route `{}` overlaps the admin URL namespace; declarative admin routes keep their own application path",
                        route.path
                    ),
                ));
            }
            if self
                .declared_routes
                .iter()
                .take(index)
                .any(|previous| paths_overlap(&previous.path, &route.path))
            {
                report.add(CheckError::new(
                    "AdminPanel",
                    format!("admin route `{}` is declared more than once", route.path),
                ));
            }
            // `Standalone` replaces nothing: any number of them per model.
            if route.overrides != AdminView::Standalone
                && self.declared_routes.iter().take(index).any(|previous| {
                    previous.model_key == route.model_key && previous.overrides == route.overrides
                })
            {
                report.add(CheckError::new(
                    "AdminPanel",
                    format!(
                        "admin view {:?} for model `{}` is overridden more than once",
                        route.overrides, route.model_key
                    ),
                ));
            }

            if self
                .declared_routes
                .iter()
                .take(index)
                .any(|previous| previous.name == route.name)
            {
                report.add(CheckError::new(
                    "AdminPanel",
                    format!(
                        "admin route name `{}` is declared more than once",
                        route.name
                    ),
                ));
            }
        }
        for error in self.site.check() {
            report.add(CheckError::new("AdminPanel", error));
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

#[cfg(test)]
mod declared_route_tests {
    use super::{AdminStaging, DeclaredAdminRoute, paths_overlap};
    use crate::admin::router::AdminView;

    #[test]
    fn overlapping_paths() {
        for (a, b) in [
            ("/a/b", "/a/b"),
            ("/a/{id}", "/a/42"),
            ("/a/42", "/a/{id}"),
            ("/a/{x}", "/a/{id}"),
            ("/a/{*rest}", "/a/b/c"),
            ("/a/b/c", "/a/{*rest}"),
            ("a/b/", "/a/b"),
        ] {
            assert!(paths_overlap(a, b), "{a} / {b}");
        }
        for (a, b) in [
            ("/a/b", "/a/c"),
            ("/a/b", "/a/b/c"),
            ("/a/b/c", "/a/b"),
            ("/a/{id", "/a/42"),
            ("/a/42", "/a/id}"),
        ] {
            assert!(!paths_overlap(a, b), "{a} / {b}");
        }
    }

    /// The accounts admin (`users`) is always registered.
    fn staging(routes: &[(&str, &str, AdminView)], public: &[&str]) -> AdminStaging {
        let mut s = AdminStaging::new();
        s.enabled = true;
        s.public_route_paths = public.iter().map(|p| p.to_string()).collect();
        s.declared_routes = routes
            .iter()
            .map(|(path, name, overrides)| DeclaredAdminRoute {
                path: path.to_string(),
                name: name.to_string(),
                model_key: "users".to_string(),
                handler: axum::routing::get(|| async { "" }),
                overrides: *overrides,
            })
            .collect();
        s
    }

    fn errors(routes: &[(&str, &str, AdminView)], public: &[&str]) -> String {
        staging(routes, public)
            .validate()
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default()
    }

    use AdminView::{Detail, Standalone};

    #[test]
    fn valid_declarations_pass() {
        assert_eq!(
            errors(
                &[
                    ("/accounts/{id}", "a", Detail),
                    ("/report", "b", Standalone),
                    ("/stats", "c", Standalone),
                    ("/administrator", "d", Standalone),
                ],
                &["/other"],
            ),
            ""
        );
    }

    #[test]
    fn a_path_must_be_absolute_without_trailing_slash() {
        for path in ["report", "/report/"] {
            assert!(
                errors(&[(path, "a", Standalone)], &[]).contains("must be an absolute path"),
                "{path}"
            );
        }
    }

    #[test]
    fn a_detail_override_needs_exactly_one_id() {
        for path in [
            "/accounts",
            "/accounts/{id}/{id}",
            "/accounts/{id}/tab/{tab}",
            "/accounts/{x}",
        ] {
            assert!(
                errors(&[(path, "a", Detail)], &[]).contains("exactly one `{id}`"),
                "{path}"
            );
        }
    }

    #[test]
    fn a_public_route_on_the_same_path_is_refused() {
        assert!(
            errors(&[("/report/{id}", "a", Standalone)], &["/report/{x}"])
                .contains("overlaps a public route")
        );
    }

    #[test]
    fn the_admin_namespace_is_refused() {
        for path in ["/admin", "/admin/report"] {
            assert!(
                errors(&[(path, "a", Standalone)], &[]).contains("admin URL namespace"),
                "{path}"
            );
        }
    }

    #[test]
    fn the_same_path_twice_is_refused() {
        assert!(
            errors(
                &[("/report", "a", Standalone), ("/report", "b", Standalone)],
                &[]
            )
            .contains("declared more than once")
        );
    }

    #[test]
    fn a_view_overridden_twice_is_refused() {
        assert!(
            errors(
                &[
                    ("/accounts/{id}", "a", Detail),
                    ("/people/{id}", "b", Detail)
                ],
                &[]
            )
            .contains("overridden more than once")
        );
    }

    #[test]
    fn declared_routes_need_the_admin_enabled() {
        let mut s = staging(&[("/report", "a", Standalone)], &[]);
        s.enabled = false;
        let error = s
            .validate()
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(error.contains("the admin is disabled"), "{error}");

        let mut s = staging(&[], &[]);
        s.enabled = false;
        assert!(s.validate().is_ok(), "nothing declared, nothing to refuse");
    }

    #[test]
    fn the_same_name_twice_is_refused() {
        assert!(
            errors(
                &[("/report", "a", Standalone), ("/stats", "a", Standalone)],
                &[]
            )
            .contains("admin route name `a` is declared more than once")
        );
    }
}
