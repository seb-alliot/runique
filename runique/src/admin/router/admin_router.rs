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
use crate::admin::registry::AdminRegistry;
use crate::app::staging::AdminStaging;
use crate::context::template::Request;
use crate::middleware::auth::{load_user_middleware, login_user_full};
use crate::urlpatterns;
use crate::utils::aliases::AppResult;
use crate::{admin::config::AdminConfig, flash_now};

#[derive(Clone)]
pub struct AdminState {
    pub registry: Arc<AdminRegistry>,
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
    let registry = admin_staging.registry;
    let config = admin_staging.config;

    let admin_state = Arc::new(AdminState {
        registry: Arc::new(registry),
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
    public_router
        .merge(
            protected_router
                .merge(generated_router)
                .layer(middleware::from_fn(admin_required)),
        )
        .layer(middleware::from_fn(load_user_middleware))
        .layer(Extension(admin_state))
}

async fn admin_dashboard_redirect() -> Response {
    Redirect::permanent("/admin/").into_response()
}

async fn admin_dashboard(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
) -> AppResult<Response> {
    req = req
        .insert("site_title", &admin.config.site_title)
        .insert("resources", &admin.registry.resources)
        .insert("current_page", "dashboard")
        .insert("current_resource", &Option::<String>::None);

    req.render("dashboard")
}

async fn admin_login_get(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
) -> AppResult<Response> {
    req = req.insert("site_title", &admin.config.site_title);
    req.render("login")
}

async fn admin_login_post(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Form(data): Form<AdminLoginData>,
) -> Response {
    let Some(auth) = &admin.config.auth else {
        return (
            StatusCode::NOT_IMPLEMENTED,
            "Aucun handler d'authentification configuré. Appelez .auth(MyAuth) sur AdminConfig.",
        )
            .into_response();
    };

    let result = auth
        .authenticate(&data.username, &data.password, &req.engine.db)
        .await;

    match result {
        Some(user) => {
            if login_user_full(
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
                req = req
                    .insert("site_title", &admin.config.site_title)
                    .insert("error", "Erreur lors de l'ouverture de session.");
                return req
                    .render("admin/login")
                    .unwrap_or_else(|e| e.into_response());
            }

            Redirect::to(&format!("{}/", admin.config.prefix)).into_response()
        }

        None => {
            req = req
                .insert("site_title", &admin.config.site_title)
                .insert("error", "Identifiants incorrects ou droits insuffisants.");
            req.render("admin/login")
                .unwrap_or_else(|e| e.into_response())
        }
    }
}

async fn admin_logout(session: Session, Extension(admin): Extension<Arc<AdminState>>) -> Response {
    let _ = session
        .insert("flash_messages", "Déconnexion réussie.")
        .await;
    let _ = session.delete().await;
    flash_now!( success => "Déconnexion réussie.");
    Redirect::to(&format!("{}/login", admin.config.prefix)).into_response()
}
