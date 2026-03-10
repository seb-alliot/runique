// ═══════════════════════════════════════════════════════════════
// Admin Router — Génération des routes CRUD + connexion à l'Engine
// ═══════════════════════════════════════════════════════════════

use std::sync::Arc;

use axum::{
    extract::Form,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Extension, Router,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::admin::middleware::admin_required;
use crate::admin::trad::insert_admin_messages;
use crate::admin::PrototypeAdminState;
use crate::app::staging::AdminStaging;
use crate::context::template::Request;
use crate::middleware::auth::{load_user_middleware, login_staff};
use crate::urlpatterns;
use crate::utils::aliases::AppResult;
use crate::utils::trad::t;
use crate::{admin::config::AdminConfig, flash_now};

#[derive(Clone)]
pub struct AdminState {
    pub config: Arc<AdminConfig>,
}

#[derive(Deserialize)]
struct AdminLoginData {
    username: String,
    password: String,
}

pub fn build_admin_router(admin_staging: AdminStaging) -> Router {
    let prefix = admin_staging
        .config
        .prefix
        .trim_end_matches('/')
        .to_string();
    let config = admin_staging.config;
    let state = admin_staging.state;

    let admin_state = Arc::new(AdminState {
        config: Arc::new(config.clone()),
    });

    // Routes publiques (login uniquement)
    let public_router = urlpatterns! {
        &format!("{}/login", prefix) => get(admin_login_get).post(admin_login_post), name = "admin:login",
    };

    // Routes protégées (dashboard + logout)
    let protected_router = urlpatterns! {
        &format!("{}/", prefix) => get(admin_dashboard), name = "admin:dashboard",
        &prefix => get(admin_dashboard_redirect), name = "admin:dashboard_redirect",
        &format!("{}/logout", prefix) => post(admin_logout), name = "admin:logout",
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
        let merged = Arc::new(PrototypeAdminState {
            registry: state.registry.clone(),
            config: Arc::new(config),
        });
        router = router.layer(Extension(merged));
    }

    router
}

async fn admin_dashboard_redirect() -> Response {
    Redirect::permanent("/admin/").into_response()
}

async fn admin_dashboard(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    proto: Option<Extension<Arc<PrototypeAdminState>>>,
) -> AppResult<Response> {
    let db = req.engine.db.clone();

    let mut resource_counts: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();

    let resources: Vec<&crate::admin::AdminResource> = if let Some(Extension(ref state)) = proto {
        for (key, entry) in &state.registry.resources {
            if let Some(count_fn) = &entry.count_fn {
                if let Ok(n) = (count_fn)(db.clone()).await {
                    resource_counts.insert(key.clone(), n);
                }
            }
        }
        state.registry.all().map(|e| &e.meta).collect()
    } else {
        Vec::new()
    };

    insert_admin_messages(&mut req.context, "dashboard");
    insert_admin_messages(&mut req.context, "base");
    req = req
        .insert("site_title", &admin.config.site_title)
        .insert("resources", &resources)
        .insert("resource_counts", &resource_counts)
        .insert("current_page", "dashboard")
        .insert("current_resource", &Option::<String>::None);

    req.render(admin.config.templates.dashboard.resolve())
}

async fn admin_login_get(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
) -> AppResult<Response> {
    insert_admin_messages(&mut req.context, "login");

    req = req.insert("site_title", &admin.config.site_title);
    req.render(admin.config.templates.login.resolve())
}

async fn admin_login_post(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Form(data): Form<AdminLoginData>,
) -> Response {
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

    match result {
        Some(user) => {
            if login_staff(
                &req.session,
                user.user_id,
                &user.username,
                user.is_staff,
                user.is_superuser,
                user.roles,
            )
            .await
            .is_err()
            {
                insert_admin_messages(&mut req.context, "login");
                insert_admin_messages(&mut req.context, "base");
                req = req
                    .insert("site_title", &admin.config.site_title)
                    .insert("error", &t("admin.login.error_session").to_string());
                return req
                    .render(admin.config.templates.login.resolve())
                    .unwrap_or_else(|e| e.into_response());
            }

            Redirect::to(&format!("{}/", admin.config.prefix)).into_response()
        }

        None => {
            insert_admin_messages(&mut req.context, "login");
            insert_admin_messages(&mut req.context, "base");
            req = req
                .insert("site_title", &admin.config.site_title)
                .insert("error", &t("admin.login.error_credentials").to_string());
            req.render(admin.config.templates.login.resolve())
                .unwrap_or_else(|e| e.into_response())
        }
    }
}

async fn admin_logout(session: Session, Extension(admin): Extension<Arc<AdminState>>) -> Response {
    let _ = session.delete().await;
    flash_now!(success => t("admin.logout.success").to_string());
    Redirect::to(&format!("{}/login", admin.config.prefix)).into_response()
}
