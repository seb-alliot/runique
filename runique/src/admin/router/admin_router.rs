//! Router admin : construit les routes CRUD, login/logout et branche le middleware d'authentification.
use std::sync::Arc;

use axum::{
    Extension, Router,
    extract::Form,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;

use crate::app::staging::AdminStaging;
use crate::context::template::Request;
use crate::middleware::auth::{
    LoginGuard, is_admin_authenticated, load_user_middleware, login, logout,
};
use crate::middleware::security::rate_limit_middleware;
use crate::urlpatterns;
use crate::utils::{
    aliases::AppResult,
    trad::{current_lang, t, tf},
};
use crate::{
    admin::{
        PrototypeAdminState, config::AdminConfig, middleware::admin_required,
        trad::insert_admin_messages,
    },
    flash_now,
};

#[derive(Clone)]
pub struct AdminState {
    pub config: Arc<AdminConfig>,
    pub login_guard: Option<Arc<LoginGuard>>,
}

#[derive(Deserialize)]
struct AdminLoginData {
    username: String,
    password: String,
    #[serde(default)]
    csrf_token: String,
}

pub fn build_admin_router(admin_staging: AdminStaging, _db: crate::utils::aliases::ADb) -> Router {
    let prefix = admin_staging
        .config
        .prefix
        .trim_end_matches('/')
        .to_string();
    let config = admin_staging.config;
    let state = admin_staging.state;

    let login_guard = config.login_guard.clone();
    let rate_limiter = config.rate_limiter.clone();

    let admin_state = Arc::new(AdminState {
        config: Arc::new(config.clone()),
        login_guard,
    });

    // Routes publiques (login uniquement)
    let login_route = urlpatterns! {
        &format!("{prefix}/login") => get(admin_login_get).post(admin_login_post), name = "admin_login",
    };
    let public_router = if let Some(limiter) = rate_limiter {
        login_route.layer(middleware::from_fn_with_state(
            limiter,
            rate_limit_middleware,
        ))
    } else {
        login_route
    };

    // Routes protégées (dashboard + logout)
    let protected_router = urlpatterns! {
        &format!("{prefix}/") => get(admin_dashboard), name = "admin_dashboard",
        &prefix => get(admin_dashboard_redirect), name = "admin_dashboard_redirect",
        &format!("{prefix}/logout") => get(admin_logout), name = "admin_logout",
    };

    // Routes CRUD générées (protégées aussi)
    let generated_router = if let Some(router) = admin_staging.route_admin {
        router
    } else {
        Router::new()
    };

    // Assemblage : public + (protected + generated avec middleware)
    let mut router = public_router
        .merge(
            protected_router
                .merge(generated_router)
                .layer(middleware::from_fn(admin_required)),
        )
        .layer(middleware::from_fn(load_user_middleware))
        .layer(Extension(admin_state));

    if let Some(state) = state {
        // On remplace le config du proto_state par celui d'AdminStaging
        // pour que les templates configurés via .templates() soient pris en compte.
        let order = config.resource_order.clone();
        let config = Arc::new(config);

        // Unwrap l'Arc<PrototypeAdminState> pour accéder aux champs en ownership.
        // try_unwrap réussit car c'est le seul propriétaire au boot.
        let registry = match Arc::try_unwrap(state) {
            Ok(proto) => match Arc::try_unwrap(proto.registry) {
                Ok(mut reg) => {
                    if !order.is_empty() {
                        reg.reorder(&order);
                    }
                    Arc::new(reg)
                }
                Err(arc) => arc,
            },
            Err(arc) => arc.registry.clone(),
        };

        let merged = Arc::new(PrototypeAdminState { registry, config });
        router = router.layer(Extension(merged));
    }

    router
}

/// Clé de session pour la surcharge runtime du template dashboard.
///
/// Un dev peut stocker un nom de template Tera dans cette clé pour remplacer
/// temporairement le template configuré via `.with_dashboard()`.
/// Si absente ou vide, `resolve()` s'applique normalement.
///
/// ## Exemple (dans un handler custom) :
/// ```rust,ignore
/// session.insert(ADMIN_TEMPLATE_SESSION_KEY, "admin/dashboard").await?;
/// ```
pub const ADMIN_TEMPLATE_SESSION_KEY: &str = "admin_template_override";

async fn admin_dashboard_redirect() -> Response {
    Redirect::permanent("/admin/").into_response()
}

async fn admin_dashboard(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(current_user): Extension<crate::middleware::auth::CurrentUser>,
    proto: Option<Extension<Arc<PrototypeAdminState>>>,
) -> AppResult<Response> {
    let db = req.engine.db.clone();

    let mut resource_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();

    const SUPERUSER_ONLY: &[&str] = &["droits", "groupes"];
    let resources: Vec<&crate::admin::AdminResource> = if let Some(Extension(ref state)) = proto {
        for (key, entry) in &state.registry.resources {
            if let Some(count_fn) = &entry.count_fn {
                if let Ok(n) = (count_fn)(db.clone(), None).await {
                    resource_counts.insert(key.clone(), n);
                }
            }
        }
        state
            .registry
            .all()
            .filter(|e| {
                if current_user.is_superuser {
                    return true;
                }
                if SUPERUSER_ONLY.contains(&e.meta.key) {
                    return false;
                }
                current_user.can_access_resource(e.meta.key)
            })
            .map(|e| &e.meta)
            .collect()
    } else {
        Vec::new()
    };

    // Groupes ayant un droit sur chaque resource_key (depuis la DB)
    let resource_groups: std::collections::HashMap<String, Vec<String>> = {
        use crate::admin::permissions::{droit, groupe};
        use sea_orm::EntityTrait;
        let groupes: std::collections::HashMap<_, String> = groupe::Entity::find()
            .all(&*db)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|g| (g.id, g.nom))
            .collect();
        let droits = droit::Entity::find().all(&*db).await.unwrap_or_default();
        let mut map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for d in droits {
            let nom = groupes
                .get(&d.groupe_id)
                .cloned()
                .unwrap_or_else(|| d.groupe_id.to_string());
            let entry = map.entry(d.resource_key).or_default();
            if !entry.contains(&nom) {
                entry.push(nom);
            }
        }
        map
    };

    let session_override: Option<String> = req
        .session
        .get(ADMIN_TEMPLATE_SESSION_KEY)
        .await
        .unwrap_or(None);

    insert_admin_messages(&mut req.context, "dashboard");
    insert_admin_messages(&mut req.context, "base");
    req = req
        .insert("current_user", &current_user)
        .insert("site_title", &admin.config.site_title)
        .insert("site_url", &admin.config.site_url)
        .insert("resources", &resources)
        .insert("resource_groups", &resource_groups)
        .insert("resource_counts", &resource_counts)
        .insert("current_page", "dashboard")
        .insert("lang", current_lang().code())
        .insert("current_resource", &Option::<String>::None)
        .insert("admin_has_session_override", session_override.is_some());

    let template = session_override
        .as_deref()
        .unwrap_or_else(|| admin.config.templates.dashboard.resolve());

    req.render(template)
}

async fn admin_login_get(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> AppResult<Response> {
    let from_logout = params.get("from").is_some_and(|v| v == "logout");
    if !from_logout && is_admin_authenticated(&req.session).await {
        return Ok(Redirect::to(&format!("{}/", admin.config.prefix)).into_response());
    }

    insert_admin_messages(&mut req.context, "login");

    req = req
        .insert("site_title", &admin.config.site_title)
        .insert("site_url", &admin.config.site_url)
        .insert("lang", current_lang().code());
    req.render(admin.config.templates.login.resolve())
}

async fn admin_login_post(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Form(data): Form<AdminLoginData>,
) -> Response {
    use crate::utils::middleware::csrf::unmask_csrf_token;
    use subtle::ConstantTimeEq;
    if is_admin_authenticated(&req.session).await {
        return Redirect::to(&format!("{}/", admin.config.prefix)).into_response();
    }
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
        req = req
            .insert("lang", current_lang().code())
            .insert("site_title", &admin.config.site_title)
            .insert("site_url", &admin.config.site_url)
            .insert("error", t("csrf.invalid_or_missing").to_string());
        return req
            .render(admin.config.templates.login.resolve())
            .unwrap_or_else(axum::response::IntoResponse::into_response);
    }

    // Vérification du login guard (brute-force)
    if let Some(guard) = &admin.login_guard {
        let key = LoginGuard::effective_key(&data.username, "unknown");
        if guard.is_locked(&key) {
            let secs = guard.remaining_lockout_secs(&key).unwrap_or(0);
            insert_admin_messages(&mut req.context, "login");
            req = req
                .insert("lang", current_lang().code())
                .insert("site_title", &admin.config.site_title)
                .insert("site_url", &admin.config.site_url)
                .insert("error", tf("admin.login.error_locked", &[secs]));
            return req
                .render(admin.config.templates.login.resolve())
                .unwrap_or_else(axum::response::IntoResponse::into_response);
        }
    }

    let Some(auth) = &admin.config.auth else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            t("admin.access.no_auth_handler").to_string(),
        )
            .into_response();
    };

    let result = auth
        .authenticate(&data.username, &data.password, &req.engine.db)
        .await;

    if let Some(user) = result {
        if let Some(guard) = &admin.login_guard {
            let key = LoginGuard::effective_key(&data.username, "unknown");
            guard.record_success(&key);
        }

        let db_store = req
            .engine
            .session_db_store
            .read()
            .ok()
            .and_then(|g| g.as_ref().cloned());
        let exclusive = req.engine.features.exclusive_login;
        if login(
            &req.session,
            &req.engine.db,
            user.user_id,
            &user.username,
            user.is_staff,
            user.is_superuser,
            db_store.as_deref(),
            exclusive,
        )
        .await
        .is_err()
        {
            insert_admin_messages(&mut req.context, "login");
            insert_admin_messages(&mut req.context, "base");
            req = req
                .insert("lang", current_lang().code())
                .insert("site_title", &admin.config.site_title)
                .insert("site_url", &admin.config.site_url)
                .insert("error", t("admin.login.error_session").to_string());
            return req
                .render(admin.config.templates.login.resolve())
                .unwrap_or_else(axum::response::IntoResponse::into_response);
        }

        Redirect::to(&format!("{}/", admin.config.prefix)).into_response()
    } else {
        if let Some(guard) = &admin.login_guard {
            let key = LoginGuard::effective_key(&data.username, "unknown");
            guard.record_failure(&key);
        }

        insert_admin_messages(&mut req.context, "login");
        insert_admin_messages(&mut req.context, "base");
        req = req
            .insert("lang", current_lang().code())
            .insert("site_title", &admin.config.site_title)
            .insert("site_url", &admin.config.site_url)
            .insert("error", t("admin.login.error_credentials").to_string());
        req.render(admin.config.templates.login.resolve())
            .unwrap_or_else(axum::response::IntoResponse::into_response)
    }
}

async fn admin_logout(req: Request, Extension(admin): Extension<Arc<AdminState>>) -> Response {
    let session = &req.session;
    let db_store = req
        .engine
        .session_db_store
        .read()
        .ok()
        .and_then(|g| g.as_ref().cloned());
    let _ = logout(session, db_store.as_deref()).await;
    let login_url = format!("{}/login?from=logout", admin.config.prefix);
    flash_now!(info => "Vous êtes deconnecté");
    Redirect::to(&login_url).into_response()
}
