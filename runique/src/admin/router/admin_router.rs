// ═══════════════════════════════════════════════════════════════
// Admin Router — Génération des routes CRUD + connexion à l'Engine
// ═══════════════════════════════════════════════════════════════

use std::sync::Arc;

use axum::{
    extract::{Form, Path},
    http::StatusCode,
    middleware,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Extension, Router,
};
use serde::Deserialize;
use tower_sessions::Session;

use crate::admin::config::AdminConfig;
use crate::admin::middleware::admin_required;
use crate::admin::registry::AdminRegistry;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::middleware::auth::{load_user_middleware, login_user_full};
use crate::urlpatterns;
use crate::utils::aliases::AppResult;

#[derive(Clone)]
pub struct AdminState {
    pub registry: Arc<AdminRegistry>,
    pub config: Arc<AdminConfig>,
}

#[derive(Deserialize)]
struct AdminLoginData {
    username: String,
    password: String,
    #[allow(dead_code)]
    csrf_token: Option<String>,
}

pub fn build_admin_router(registry: AdminRegistry, config: AdminConfig) -> Router {
    let prefix = config.prefix.trim_end_matches('/').to_string();

    let admin_state = Arc::new(AdminState {
        registry: Arc::new(registry),
        config: Arc::new(config.clone()),
    });

    // Routes publiques (login)
    let public_router = urlpatterns! {
        &format!("{}/login", prefix) => get(admin_login_get).post(admin_login_post), name = "admin:login",
    };

    // Routes protégées (dashboard, logout)
    let protected_router = urlpatterns! {
        &format!("{}/", prefix) => get(admin_dashboard), name = "admin:dashboard",
        &prefix => get(admin_dashboard_redirect), name = "admin:dashboard_redirect",
        &format!("{}/logout", prefix) => post(admin_logout), name = "admin:logout",
    };

    // Routes CRUD dynamiques avec pattern {resource}
    let crud_router = Router::new()
        .route(&format!("{}/{{resource}}/list", prefix), get(admin_list))
        .route(
            &format!("{}/{{resource}}/create", prefix),
            get(admin_create_get).post(admin_create_post),
        )
        .route(
            &format!("{}/{{resource}}/{{id}}", prefix),
            get(admin_detail).post(admin_edit),
        )
        .route(
            &format!("{}/{{resource}}/{{id}}/delete", prefix),
            post(admin_delete),
        );

    public_router
        .merge(
            protected_router
                .merge(crud_router)
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

    req.render("admin/dashboard")
}

async fn admin_login_get(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
) -> AppResult<Response> {
    req = req.insert("site_title", &admin.config.site_title);
    req.render("admin/login")
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
    let _ = session.flush().await;
    Redirect::to(&format!("{}/login", admin.config.prefix)).into_response()
}

// ═══════════════════════════════════════════════════════════════
// Handlers CRUD
// ═══════════════════════════════════════════════════════════════

async fn admin_list(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Path(resource_key): Path<String>,
) -> AppResult<Response> {
    let resource = admin
        .registry
        .resources
        .iter()
        .find(|r| r.key == resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    req = req
        .insert("site_title", &admin.config.site_title)
        .insert("current_page", "list")
        .insert("current_resource", &resource_key)
        .insert("resource", resource)
        .insert("rows", Vec::<serde_json::Value>::new())
        .insert("columns", Vec::<String>::new())
        .insert("total", 0)
        .insert("current_page", 1)
        .insert("total_pages", 1);

    req.render("admin/list")
}

async fn admin_create_get(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Path(resource_key): Path<String>,
) -> AppResult<Response> {
    let resource = admin
        .registry
        .resources
        .iter()
        .find(|r| r.key == resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    req = req
        .insert("site_title", &admin.config.site_title)
        .insert("current_page", "list")
        .insert("current_resource", &resource_key)
        .insert("resource", resource)
        .insert("rows", Vec::<serde_json::Value>::new())
        .insert("columns", Vec::<String>::new())
        .insert("total", 0)
        .insert("current_page", 1)
        .insert("total_pages", 1)
        .insert("is_edit", false);

    req.render("admin/form")
}

async fn admin_create_post(
    Extension(admin): Extension<Arc<AdminState>>,
    Path(resource_key): Path<String>,
) -> Response {
    // TODO: Insert dans la DB
    Redirect::to(&format!("{}/{}/list", admin.config.prefix, resource_key)).into_response()
}

async fn admin_detail(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
    Path((resource_key, id)): Path<(String, i32)>,
) -> AppResult<Response> {
    let resource = admin
        .registry
        .resources
        .iter()
        .find(|r| r.key == resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    req = req
        .insert("site_title", &admin.config.site_title)
        .insert("current_page", "edit")
        .insert("current_resource", &resource_key)
        .insert("resource", resource)
        .insert("form_fields", Vec::<serde_json::Value>::new())
        .insert("is_edit", true)
        .insert("object_id", id);

    req.render("admin/form")
}

async fn admin_edit(
    Extension(admin): Extension<Arc<AdminState>>,
    Path((resource_key, id)): Path<(String, i32)>,
) -> Response {
    // TODO: Update dans la DB
    Redirect::to(&format!("{}/{}/{}", admin.config.prefix, resource_key, id)).into_response()
}

async fn admin_delete(
    Extension(admin): Extension<Arc<AdminState>>,
    Path((resource_key, __id)): Path<(String, i32)>,
) -> Response {
    // TODO: Delete dans la DB
    Redirect::to(&format!("{}/{}/list", admin.config.prefix, resource_key)).into_response()
}
