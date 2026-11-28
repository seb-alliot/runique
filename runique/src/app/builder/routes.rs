//! The routes Runique adds to the project's: the welcome page, the password
//! reset, the admin. All are merged before the middleware stack: `.layer()`
//! in Axum only covers the routes present when it's called, so a route merged
//! after would run without Session, CSRF or the engine.
#[cfg(feature = "admin")]
use crate::admin::router::routes::AdminCachePolicy;
#[cfg(feature = "admin")]
use crate::app::staging::AdminStaging;
#[cfg(feature = "admin")]
use crate::config::RuniqueConfig;
#[cfg(feature = "admin")]
use crate::errors::build::BuildError;
#[cfg(feature = "admin")]
use crate::templates::loader::Integrity;
use crate::{
    auth::{
        PasswordResetConfig, PasswordResetStaging,
        password::{build_reset_router, build_router},
    },
    http::request::Request,
    router::register_name_url,
    utils::aliases::{AEngine, AppResult},
};
#[cfg(feature = "admin")]
use crate::{auth::permissions::prune_orphan_droits, config::env::css_token, translation::t};
use axum::{Router, response::Response};
use std::sync::Arc;
#[cfg(feature = "admin")]
use tracing::Level;

/// Runique's welcome page, served at `/` in debug when the app declares no `/`.
async fn default_index(mut request: Request) -> AppResult<Response> {
    request.render("base_index.html")
}

/// In development, Runique's welcome page on a `/` the project leaves free.
/// In production, an undeclared `/` stays a 404: a welcome page on a live
/// site would look unfinished and name the framework.
pub(super) fn welcome_page(
    router: Router,
    dev: bool,
    root_free: bool,
    password_reset: Option<&PasswordResetStaging>,
) -> Router {
    let forgot_on_root = password_reset.is_some_and(|pr| pr.config.forgot_route == "/");
    if dev && root_free && !forgot_on_root {
        router.route("/", axum::routing::get(default_index))
    } else {
        router
    }
}

/// The reset page an enabled admin needs for the accounts it creates, when
/// the site declared no `with_password_reset`: served alone, with Runique's
/// template and default limits. The "forgot password" form is the site's to
/// open, by declaring it.
///
/// The admin's emailed reset links point to the reset page this app serves.
/// Its "view site" link stays `/` unless set: on the host the browser is on,
/// so the session cookie follows.
#[cfg(feature = "admin")]
pub(super) fn admin_reset_page(
    admin: &mut AdminStaging,
    password_reset: Option<&PasswordResetStaging>,
) -> Option<PasswordResetConfig> {
    let admin_reset =
        (admin.enabled && password_reset.is_none()).then(PasswordResetConfig::default);
    if let Some(pr) = password_reset {
        admin.config.reset_link = Some((&pr.config).into());
    } else if let Some(config) = &admin_reset {
        admin.config.reset_link = Some(config.into());
    }
    admin_reset
}

/// Mounts the password reset the project declared, or the reset page alone
/// the admin needs, and names their URLs.
pub(super) fn router_reset_password(
    router: Router,
    engine: &AEngine,
    password_reset: Option<PasswordResetStaging>,
    admin_reset: Option<PasswordResetConfig>,
) -> Router {
    if let Some(pr) = password_reset {
        let forgot_path = pr.config.forgot_route.clone();
        let reset_path = format!("{}/{{token}}", pr.config.reset_route.trim_end_matches('/'));
        let pr_router = build_router(Arc::new(pr.config));
        register_name_url(engine, "forgot_password", &forgot_path);
        register_name_url(engine, "reset_password", &reset_path);
        router.merge(pr_router)
    } else if let Some(config) = admin_reset {
        let reset_path = format!("{}/{{token}}", config.reset_route.trim_end_matches('/'));
        let reset_router = build_reset_router(Arc::new(config));
        register_name_url(engine, "reset_password", &reset_path);
        router.merge(reset_router)
    } else {
        router
    }
}

/// Mounts the admin, names its URLs, and serves `robots.txt` when asked.
#[cfg(feature = "admin")]
pub(super) async fn router_admin(
    router: Router,
    engine: &AEngine,
    admin: AdminStaging,
    integrity: &Integrity,
) -> Result<Router, BuildError> {
    use crate::router::add_urls;
    if !admin.enabled {
        return Ok(router);
    }
    // Public base URL (mount prefix + admin path) — what `{% url %}`
    // must resolve to, not the path the router is built from.
    let admin_prefix = admin.public_prefix();
    let robots_txt = admin.robots_txt;
    let sitemap_url = admin.sitemap_url.clone();
    let declared_route_urls: Vec<(String, String)> = admin
        .declared_routes
        .iter()
        .map(|route| (route.name.clone(), route.path.clone()))
        .collect();
    admin.log_boot(engine.config.dev);
    drop_unregistered_rights(engine, &admin).await;

    let sheets = admin_sheets(&admin, &engine.config, integrity)?;
    let admin_router = crate::admin::build_admin_router(admin, sheets);
    add_urls(engine);
    register_name_url(engine, "admin", &admin_prefix);
    for (name, path) in declared_route_urls {
        let already_registered = engine
            .url_registry
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(&name);
        if already_registered {
            return Err(BuildError::validation(format!(
                "admin route name `{name}` conflicts with an already registered URL"
            )));
        }
        register_name_url(engine, name, path);
    }
    let router = router.merge(admin_router);
    Ok(if robots_txt {
        with_robots_txt(router, sitemap_url)
    } else {
        router
    })
}

/// Closes the `resource_key` referential-integrity gap: drops the rights that
/// point at a resource no longer registered (see `prune_orphan_droits`). The
/// registered models are fixed at build time: boot is the only window. Only
/// once the project registered its models: the accounts alone would make
/// every other right look orphaned.
#[cfg(feature = "admin")]
async fn drop_unregistered_rights(engine: &AEngine, admin: &AdminStaging) {
    let valid_keys = admin.site.keys();
    if !admin.site.has_project_models() {
        return;
    }
    let level = crate::log::log_level!(builder.registry);
    match prune_orphan_droits(&engine.db, &valid_keys).await {
        Ok(n) if n > 0 => {
            if let Some(level) = level {
                crate::runique_log!(level, pruned = n, "orphan group rights pruned at boot");
            }
        }
        Ok(_) => {}
        Err(e) => {
            // Never silent: stale grants would stay without a word.
            crate::runique_log!(
                level.unwrap_or(Level::WARN),
                error = %e,
                "{}", t(runique_i18n::keys::build::log::PRUNE_FAILED)
            );
        }
    }
}

/// The stylesheets of each admin page, with their integrity hashes outside
/// development.
#[cfg(feature = "admin")]
fn admin_sheets(
    admin: &AdminStaging,
    config: &RuniqueConfig,
    integrity: &Integrity,
) -> Result<crate::admin::template::PageSheets, BuildError> {
    let statics = &config.static_files;
    let version = css_token();
    let sheets = crate::admin::template::PageSheets::build(
        &admin.config.templates,
        &admin.site.model_templates(),
        &crate::admin::template::stylesheet::Statics {
            site_url: &statics.static_url,
            site: &integrity.site,
            runique_url: &statics.static_runique_url,
            runique: &integrity.runique,
            version: &version,
        },
    )
    .map_err(BuildError::template)?;
    Ok(if config.dev {
        sheets.without_integrity()
    } else {
        sheets
    })
}

#[cfg(feature = "admin")]
fn with_robots_txt(router: Router, sitemap_url: Option<String>) -> Router {
    router.route(
        "/robots.txt",
        axum::routing::get(move || {
            let sitemap_line = sitemap_url
                .map(|u| format!("Sitemap: {}\n", u))
                .unwrap_or_default();
            // No `Disallow` for the admin prefix on purpose: this file
            // is public, so naming the path would publish the very
            // prefix a dev may have chosen to stay discreet — and it
            // would only ask crawlers not to fetch it, not to leave it
            // out of the index. De-indexing is handled where it cannot
            // leak, by the `X-Robots-Tag` header on the admin router.
            let body = format!(
                "User-agent: *\n\n{}Content-Signal: ai-train=yes, search=yes, ai-input=yes\n",
                sitemap_line
            );
            async move { body }
        }),
    )
}

/// What the admin's `no-store` policy covers, read before the admin is
/// consumed by its router.
#[cfg(feature = "admin")]
pub(super) fn admin_cache_policy(admin: &AdminStaging) -> Option<AdminCachePolicy> {
    admin.enabled.then(|| {
        AdminCachePolicy::new(
            admin.public_prefix(),
            admin
                .declared_routes
                .iter()
                .map(|route| route.path.clone())
                .collect(),
        )
    })
}

/// `Cache-Control: no-store` on the admin's responses, outermost.
#[cfg(feature = "admin")]
pub(super) fn no_store_on_admin(router: Router, policy: Option<AdminCachePolicy>) -> Router {
    match policy {
        Some(policy) => router.layer(axum::middleware::from_fn_with_state(
            policy,
            crate::admin::router::routes::admin_no_store,
        )),
        None => router,
    }
}

/// Traces how many named URLs the registry holds once every route is in.
pub(super) fn log_url_registry(engine: &AEngine) {
    if let Some(level) = crate::log::log_level!(builder.routes) {
        let count = engine
            .url_registry
            .read()
            .map(|guard| guard.len())
            .unwrap_or(0);
        crate::runique_log!(level, routes = count, "url registry");
    }
}
