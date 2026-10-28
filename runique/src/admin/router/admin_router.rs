//! Admin router: builds CRUD routes, login/logout, and attaches authentication middleware.
use crate::utils::config::TraceResult;
use std::sync::Arc;

use axum::{
    Extension, Router,
    extract::{Request as AxumRequest, State},
    http::{HeaderName, HeaderValue},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use serde::Deserialize;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::admin::builder::site::AdminSite;
use crate::app::staging::AdminStaging;
use crate::auth::{
    guard::LoginGuard,
    session::{login, logout},
};
use crate::context::template::Request;
use crate::middleware::security::rate_limit_middleware;
use crate::urlpatterns;
use crate::utils::{
    aliases::{AppResult, StrMap},
    trad::{current_lang, t, tf},
};
use crate::{
    admin::{
        config::AdminConfig,
        middleware::{admin_access, declared_route_access},
        trad::{inject_admin_prefix, insert_admin_messages},
    },
    flash_now,
};

/// Shared admin state carried in an Axum `Extension`: the resolved config
/// (with the public URL prefix already applied) and the optional login guard
/// used by the login route to throttle brute-force attempts.
#[derive(Clone)]
pub struct AdminState {
    pub config: Arc<AdminConfig>,
    pub login_guard: Option<Arc<LoginGuard>>,
    /// Every admin route's path, under the public prefix.
    pub paths: crate::admin::router::AdminPaths,
}

#[derive(Clone)]
pub(crate) struct AdminCachePolicy {
    admin_prefix: String,
    declared_paths: Vec<String>,
}

impl AdminCachePolicy {
    pub(crate) fn new(admin_prefix: String, declared_paths: Vec<String>) -> Self {
        Self {
            admin_prefix: admin_prefix.trim_end_matches('/').to_string(),
            declared_paths,
        }
    }

    fn protects(&self, path: &str) -> bool {
        (!self.admin_prefix.is_empty()
            && (path == self.admin_prefix || path.starts_with(&format!("{}/", self.admin_prefix))))
            || self
                .declared_paths
                .iter()
                .any(|pattern| route_pattern_matches(pattern, path))
    }
}

fn route_pattern_matches(pattern: &str, path: &str) -> bool {
    let pattern_segments = pattern.split('/').filter(|segment| !segment.is_empty());
    let path_segments = path.split('/').filter(|segment| !segment.is_empty());
    let mut path_segments = path_segments;

    for segment in pattern_segments {
        if segment.starts_with("{*") && segment.ends_with('}') {
            return true;
        }
        let Some(path_segment) = path_segments.next() else {
            return false;
        };
        if !(segment.starts_with('{') && segment.ends_with('}') || segment == path_segment) {
            return false;
        }
    }

    path_segments.next().is_none()
}

pub(crate) async fn admin_no_store(
    State(policy): State<AdminCachePolicy>,
    request: AxumRequest,
    next: Next,
) -> Response {
    let protect = policy.protects(request.uri().path());
    let mut response = next.run(request).await;
    if protect {
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            HeaderValue::from_static("no-store"),
        );
    }
    response
}

#[cfg(test)]
mod cache_policy_tests {
    use super::AdminCachePolicy;

    #[test]
    fn protects_admin_prefix_and_declared_route_patterns() {
        let policy = AdminCachePolicy::new(
            "/secret/admin".to_string(),
            vec![
                "/view-user".to_string(),
                "/reports/{id}/detail".to_string(),
                "/literal/{id".to_string(),
            ],
        );

        for path in [
            "/secret/admin",
            "/secret/admin/users/list",
            "/view-user",
            "/reports/42/detail",
        ] {
            assert!(policy.protects(path), "{path}");
        }

        for path in [
            "/secret/administrator",
            "/public",
            "/view-user/extra",
            "/reports/42/detail/extra",
            "/literal/42",
        ] {
            assert!(!policy.protects(path), "{path}");
        }
    }
}

#[derive(Deserialize)]
struct AdminLoginData {
    username: String,
    password: String,
    #[serde(default)]
    csrf_token: String,
}

/// Builds the complete admin Axum router: public login routes, protected
/// dashboard/history/logout routes, and the daemon-generated CRUD router —
/// all mounted under the resolved public prefix (`mount_prefix` + admin's own
/// `path`) and wrapped with the auth-required, session-loading and
/// `X-Robots-Tag: noindex` layers.
pub fn build_admin_router(admin_staging: AdminStaging) -> Router {
    // Two independent values, never in competition:
    //   `admin_path`   — the admin's own path (`/site-admin`), from `.routes()`
    //   `mount_prefix` — the segment `.prefix()` puts in front (`/secret`)
    //   `prefix`       — the public base (`/secret/site-admin`), what a visitor types
    //
    // Every admin route is declared through `urlpatterns!`, which registers its
    // name in the URL registry with the literal path given here — so they are
    // all built with the PUBLIC base, otherwise `{% url "admin_login" %}` or
    // `{% link "admin_detail" … %}` resolves to a path the router doesn't serve.
    let admin_path = admin_staging
        .config
        .prefix
        .trim_end_matches('/')
        .to_string();
    let mount_prefix = admin_staging.mount_prefix.clone();
    let prefix = format!("{mount_prefix}{admin_path}");
    let mut config = admin_staging.config;
    config.prefix = prefix.clone();

    let login_guard = config.login_guard.clone();
    let rate_limiter = config.rate_limiter.clone();

    if let Some(limiter) = &rate_limiter {
        limiter.spawn_cleanup(limiter.cleanup_interval());
    }
    if let Some(guard) = &login_guard {
        guard.spawn_cleanup(guard.cleanup_interval());
    }

    let paths = crate::admin::router::AdminPaths::new(&prefix);
    let admin_state = Arc::new(AdminState {
        config: Arc::new(config.clone()),
        login_guard,
        paths: paths.clone(),
    });

    // Public routes (login only)
    let login_get_route = urlpatterns! {
        &paths.login => get(admin_login_get), name = "admin_login",
    };

    let login_post_route = Router::new().route(&paths.login, post(admin_login_post));

    let login_post_route = if let Some(limiter) = rate_limiter {
        login_post_route.layer(middleware::from_fn_with_state(
            limiter,
            rate_limit_middleware,
        ))
    } else {
        login_post_route
    };

    let public_router = login_get_route.merge(login_post_route).into_router();

    // Protected routes (dashboard + logout)
    let protected_router = urlpatterns! {
        &paths.dashboard => get(admin_dashboard), name = "admin_dashboard",
        &paths.dashboard_redirect => get(admin_dashboard_redirect), name = "admin_dashboard_redirect",
        &paths.logout => post(admin_logout), name = "admin_logout",
        &paths.toggle_template => get(admin_toggle_template), name = "admin_toggle_template",
    }
    .into_router();

    // Generated pages live under the admin mount. `urlpatterns!` admin routes
    // keep their declared paths and get their own boundary below.
    let declared_routes = admin_staging.declared_routes;
    let mut site = admin_staging.site;
    for route in &declared_routes {
        if route.overrides == crate::admin::router::AdminView::Detail {
            site.override_view(route.model_key.clone(), route.overrides, route.path.clone());
        }
    }
    let site = Arc::new(site);
    let generated_router = crate::admin::router::refonte_router(&paths, site.clone());
    // Outside `load_admin_user` and `admin_access`: at the site's own paths,
    // a refusal must neither reveal the admin's prefix nor close the session.
    // `route_layer`: an unmatched path stays a 404 instead of a redirect. It
    // panics on a router without routes, hence the guard.
    let declared_router = if declared_routes.is_empty() {
        Router::new()
    } else {
        declared_routes
            .into_iter()
            .fold(Router::new(), |router, route| {
                router.route(&route.path, route.handler)
            })
            .route_layer(middleware::from_fn(declared_route_access))
            .route_layer(Extension(site.clone()))
    };

    // Assembly: (public + (protected + generated) behind the admin's own
    // middlewares) + declared routes behind theirs.
    public_router
        .merge(
            protected_router
                .merge(generated_router)
                .layer(middleware::from_fn(admin_access))
                .layer(Extension(site)),
        )
        .layer(middleware::from_fn(
            crate::admin::middleware::admin_middleware::load_admin_user,
        ))
        .merge(declared_router)
        .layer(Extension(admin_state))
        // Keep the admin out of search indexes without ever naming its path in a
        // public file: a `Disallow` in robots.txt would publish the very prefix a
        // dev picked to stay discreet, and it only asks crawlers not to *fetch* —
        // a linked URL can still be listed. This header says "do not index" and
        // applies to every admin response, login page included (it sits above
        // `admin_access`, which covers protected routes only), so redirects,
        // errors and JSON are covered too. `overriding`: no admin page has a
        // legitimate reason to be indexed, so no handler may weaken it.
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-robots-tag"),
            HeaderValue::from_static("noindex, nofollow"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-store"),
        ))
}

/// Session key for runtime override of the dashboard template.
///
/// A developer can store a Tera template name in this key to temporarily replace
/// the template configured via `.with_dashboard()`.
/// If absent or empty, `resolve()` applies normally.
///
/// ## Example (in a custom handler):
/// ```rust,ignore
/// session.insert(ADMIN_TEMPLATE_SESSION_KEY, "admin/dashboard").await?;
/// ```
pub const ADMIN_TEMPLATE_SESSION_KEY: &str = "admin_template_override";

async fn admin_dashboard_redirect(Extension(admin): Extension<Arc<AdminState>>) -> Response {
    Redirect::permanent(&admin.paths.dashboard).into_response()
}

async fn admin_dashboard(
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(current_user): Extension<crate::auth::current_user::CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    // One query for every count; a failure leaves the counts out, traced.
    let resource_counts = site
        .counts(&req.engine.db, &current_user)
        .await
        .trace(
            crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.list),
            "dashboard counts",
        )
        .unwrap_or_default();

    let session_override: Option<String> = req
        .session
        .get(ADMIN_TEMPLATE_SESSION_KEY)
        .await
        .unwrap_or(None);

    // The menu, rights applied, as on every page; its resources list the
    // models the dashboard counts.
    super::chrome(&mut req, &admin, &site, &current_user, "");
    insert_admin_messages(&mut req.context, "dashboard");
    req = req
        .insert("resource_counts", &resource_counts)
        .insert("current_page", "dashboard")
        .insert("admin_has_session_override", session_override.is_some());

    let template = session_override
        .as_deref()
        .unwrap_or_else(|| admin.config.templates.dashboard.resolve());

    req.render(template)
}

/// `load_admin_user` (outer layer) puts the account here only while the
/// database still lets it into the admin: the session alone vouches for nothing.
fn signed_in_admin(
    current_user: Option<&Extension<crate::auth::current_user::CurrentUser>>,
) -> bool {
    current_user.is_some_and(|Extension(user)| user.can_access_admin())
}

async fn admin_login_get(
    Extension(admin): Extension<Arc<AdminState>>,
    axum::extract::Query(params): axum::extract::Query<StrMap>,
    current_user: Option<Extension<crate::auth::current_user::CurrentUser>>,
    mut req: Request,
) -> AppResult<Response> {
    let from_logout = params.get("from").is_some_and(|v| v == "logout");
    if !from_logout && signed_in_admin(current_user.as_ref()) {
        return Ok(Redirect::to(&admin.paths.dashboard).into_response());
    }

    insert_admin_messages(&mut req.context, "login");
    inject_admin_prefix(&mut req.context, &admin.config.prefix);

    req = req
        .insert("site_title", &admin.config.site_title)
        .insert("site_url", admin.config.view_site_href())
        .insert("lang", current_lang().code());
    req.render(admin.config.templates.login.resolve())
}

async fn admin_login_post(
    Extension(admin): Extension<Arc<AdminState>>,
    current_user: Option<Extension<crate::auth::current_user::CurrentUser>>,
    client_ip: Option<Extension<crate::middleware::security::trusted_proxies::ClientIp>>,
    mut req: Request,
) -> Response {
    use crate::utils::crypto::csrf::unmask_csrf_token;
    // Keys the guard's anonymous attempts (empty username) per client, not in one bucket.
    let ip = client_ip.map_or_else(|| "unknown".to_string(), |Extension(c)| c.0.to_string());
    use subtle::ConstantTimeEq;
    if signed_in_admin(current_user.as_ref()) {
        return Redirect::to(&admin.paths.dashboard).into_response();
    }
    // Fail fast under memory pressure: a saturated session store would refuse the new
    // session at commit time and surface as a generic 500. Return a clean 503 instead.
    if req.engine.session_store_saturated() {
        return axum::http::StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let data = AdminLoginData {
        username: req.prisme.data.get("username").cloned().unwrap_or_default(),
        password: req.prisme.data.get("password").cloned().unwrap_or_default(),
        csrf_token: req
            .prisme
            .data
            .get(crate::utils::constante::session_key::session::CSRF_TOKEN_KEY)
            .cloned()
            .unwrap_or_default(),
    };
    let csrf_valid = unmask_csrf_token(&data.csrf_token)
        .map(|unmasked| {
            bool::from(
                unmasked
                    .as_bytes()
                    .ct_eq(req.csrf_token.as_str().as_bytes()),
            )
        })
        .unwrap_or(false);
    if !csrf_valid {
        insert_admin_messages(&mut req.context, "login");
        inject_admin_prefix(&mut req.context, &admin.config.prefix);
        req = req
            .insert("lang", current_lang().code())
            .insert("site_title", &admin.config.site_title)
            .insert("site_url", admin.config.view_site_href())
            .insert("error", t("csrf.invalid_or_missing").to_string());
        return req
            .render(admin.config.templates.login.resolve())
            .unwrap_or_else(axum::response::IntoResponse::into_response);
    }

    // Login guard verification (brute-force)
    if let Some(guard) = &admin.login_guard {
        let key = LoginGuard::effective_key(&data.username, &ip);
        if guard.is_locked(&key) {
            let secs = guard.remaining_lockout_secs(&key).unwrap_or(0);
            insert_admin_messages(&mut req.context, "login");
            inject_admin_prefix(&mut req.context, &admin.config.prefix);
            req = req
                .insert("lang", current_lang().code())
                .insert("site_title", &admin.config.site_title)
                .insert("site_url", admin.config.view_site_href())
                .insert("error", tf("admin.login.error_locked", &[secs]));
            return req
                .render(admin.config.templates.login.resolve())
                .unwrap_or_else(axum::response::IntoResponse::into_response);
        }
    }

    let result =
        crate::auth::authenticate_admin(&req.engine.db, &data.username, &data.password).await;

    if let Some(user) = result {
        if let Some(guard) = &admin.login_guard {
            let key = LoginGuard::effective_key(&data.username, &ip);
            guard.record_success(&key);
        }

        let db_store = req
            .engine
            .session_db_store
            .read()
            .ok()
            .and_then(|g| g.as_ref().cloned());
        let exclusive = req.engine.features.exclusive_login;
        if login(&req.session, &user, db_store.as_deref(), exclusive)
            .await
            .is_err()
        {
            insert_admin_messages(&mut req.context, "login");
            insert_admin_messages(&mut req.context, "base");
            inject_admin_prefix(&mut req.context, &admin.config.prefix);
            req = req
                .insert("lang", current_lang().code())
                .insert("site_title", &admin.config.site_title)
                .insert("site_url", admin.config.view_site_href())
                .insert("error", t("admin.login.error_session").to_string());
            return req
                .render(admin.config.templates.login.resolve())
                .unwrap_or_else(axum::response::IntoResponse::into_response);
        }

        Redirect::to(&admin.paths.dashboard).into_response()
    } else {
        if let Some(guard) = &admin.login_guard {
            let key = LoginGuard::effective_key(&data.username, &ip);
            guard.record_failure(&key);
        }

        insert_admin_messages(&mut req.context, "login");
        insert_admin_messages(&mut req.context, "base");
        inject_admin_prefix(&mut req.context, &admin.config.prefix);
        req = req
            .insert("lang", current_lang().code())
            .insert("site_title", &admin.config.site_title)
            .insert("site_url", admin.config.view_site_href())
            .insert("error", t("admin.login.error_credentials").to_string());
        req.render(admin.config.templates.login.resolve())
            .unwrap_or_else(axum::response::IntoResponse::into_response)
    }
}

async fn admin_toggle_template(
    Extension(admin): Extension<Arc<AdminState>>,
    req: Request,
) -> Response {
    let current: Option<String> = req
        .session
        .get::<String>(ADMIN_TEMPLATE_SESSION_KEY)
        .await
        .unwrap_or(None);
    let session_level = crate::utils::runique_log::get_log()
        .session
        .as_ref()
        .and_then(|s| s.store);
    if current.is_some() {
        req.session
            .remove::<String>(ADMIN_TEMPLATE_SESSION_KEY)
            .await
            .trace(session_level, "remove admin template session key");
    } else {
        req.session
            .insert(ADMIN_TEMPLATE_SESSION_KEY, "admin/dashboard.html")
            .await
            .trace(session_level, "insert admin template session key");
    }
    Redirect::to(&admin.paths.dashboard).into_response()
}

async fn admin_logout(Extension(admin): Extension<Arc<AdminState>>, req: Request) -> Response {
    let session = &req.session;
    let db_store = req
        .engine
        .session_db_store
        .read()
        .ok()
        .and_then(|g| g.as_ref().cloned());
    logout(session, db_store.as_deref()).await.trace(
        crate::utils::runique_log::get_log()
            .session
            .as_ref()
            .and_then(|s| s.store),
        "admin logout",
    );
    let login_url = format!("{}?from=logout", admin.paths.login);
    flash_now!(info => t("admin.logout_success").as_ref());
    Redirect::to(&login_url).into_response()
}
