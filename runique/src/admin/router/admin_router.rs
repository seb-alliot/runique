// ═══════════════════════════════════════════════════════════════
// Admin Router — Génération des routes CRUD + connexion à l'Engine
// ═══════════════════════════════════════════════════════════════
//
// Architecture :
//
//   AdminState (Arc) injecté comme Extension sur le router admin.
//   Le Request extractor (context/template.rs) récupère l'engine
//   depuis les extensions injectées par le middleware slot 0.
//
//   Handlers authentifiés utilisent :
//     - `req: Request`            → engine, csrf_token, session, tera
//     - `Extension(admin)`        → registry, config
//
//   Routes générées pour chaque ressource (ex: "users") :
//     GET  /admin/users/list         → liste paginée
//     GET  /admin/users/create       → formulaire création
//     POST /admin/users/create       → traitement création
//     GET  /admin/users/:id          → détail / formulaire édition
//     POST /admin/users/:id          → traitement édition
//     POST /admin/users/:id/delete   → suppression (POST pour CSRF)
//
//   Note : Les handlers CRUD sont des stubs jusqu'à ce que le daemon
//   génère les vrais handlers typés dans target/runique/admin/generated.rs.
// ═══════════════════════════════════════════════════════════════

use std::sync::Arc;

use axum::{
    extract::Path,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Extension, Router,
};
use tower_sessions::Session;

use crate::admin::config::AdminConfig;
use crate::admin::middleware::admin_required;
use crate::admin::registry::AdminRegistry;
use crate::context::template::Request;
use crate::utils::aliases::AppResult;

// ───────────────────────────────────────────────────────────────
// AdminState — Contexte admin partagé entre les handlers
// ───────────────────────────────────────────────────────────────
//
// Injecté comme `Extension(Arc<AdminState>)` sur le router admin.
// Accessible dans les handlers via :
//   `Extension(admin): Extension<Arc<AdminState>>`

#[derive(Clone)]
pub struct AdminState {
    pub registry: Arc<AdminRegistry>,
    pub config: Arc<AdminConfig>,
}

// ───────────────────────────────────────────────────────────────
// Builder de routes admin
// ───────────────────────────────────────────────────────────────

/// Construit le Router admin complet depuis l'AdminRegistry et l'AdminConfig.
///
/// Les handlers authentifiés accèdent à l'engine via le `Request` extractor
/// (injecté par le middleware d'extensions, slot 0 du MiddlewareStaging).
/// Les données admin (registry, config) sont disponibles via `Extension<Arc<AdminState>>`.
///
/// Structure des routes :
///   - Routes protégées : middleware `admin_required` appliqué
///   - Route login : publique
pub fn build_admin_router(registry: AdminRegistry, config: AdminConfig) -> Router {
    let prefix = config.prefix.clone();

    let admin_state = Arc::new(AdminState {
        registry: Arc::new(registry),
        config: Arc::new(config),
    });

    // Routes CRUD générées dynamiquement depuis le registry
    let mut crud_router = Router::new();
    for resource in &admin_state.registry.resources {
        let key = resource.key;
        crud_router = crud_router
            // Liste
            .route(&format!("/{}/list", key), get(admin_stub_list))
            // Création
            .route(
                &format!("/{}/create", key),
                get(admin_stub_create_get).post(admin_stub_create_post),
            )
            // Détail / édition
            .route(
                &format!("/{key}/{{id}}"),
                get(admin_stub_detail).post(admin_stub_edit),
            )
            // Suppression
            .route(&format!("/{key}/{{id}}/delete"), post(admin_stub_delete));
    }

    // Routes protégées — `admin_required` appliqué à toutes
    let protected = Router::new()
        .route("/", get(admin_dashboard))
        .route("/logout", post(admin_logout))
        .merge(crud_router)
        .layer(middleware::from_fn(admin_required));

    // Routes publiques — pas d'auth requise
    let public = Router::new().route("/login", get(admin_login_get).post(admin_login_post));

    // Assemblage final avec préfixe + injection de l'AdminState
    Router::new()
        .nest(&prefix, protected.merge(public))
        .layer(Extension(admin_state))
}

// ───────────────────────────────────────────────────────────────
// Handlers — Interface admin
// ───────────────────────────────────────────────────────────────

/// Tableau de bord admin
///
/// Affiche la liste des ressources enregistrées avec leurs liens CRUD.
async fn admin_dashboard(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
) -> AppResult<Response> {
    req = req
        .insert("site_title", &admin.config.site_title)
        .insert("resources", &admin.registry.resources)
        .insert("current_page", "dashboard");

    req.render("admin/dashboard")
}

/// Page de connexion admin (GET)
async fn admin_login_get(
    mut req: Request,
    Extension(admin): Extension<Arc<AdminState>>,
) -> AppResult<Response> {
    req = req.insert("site_title", &admin.config.site_title);
    req.render("admin/login")
}

/// Traitement connexion admin (POST)
///
/// TODO : Implémenter la validation username/password via le modèle User.
async fn admin_login_post() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Authentification non encore implémentée",
    )
        .into_response()
}

/// Déconnexion admin — flush la session et redirige vers login
async fn admin_logout(session: Session) -> Response {
    let _ = session.flush().await;
    Redirect::to("/admin/login").into_response()
}

// ───────────────────────────────────────────────────────────────
// Handlers stubs CRUD (remplacés par le daemon)
// ───────────────────────────────────────────────────────────────
//
// Ces handlers sont des placeholders fonctionnels.
// Le daemon génère les vrais handlers typés dans
// target/runique/admin/generated.rs lors de l'exécution de
// `runique start` ou `runique run`.

async fn admin_stub_list() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération — lancez `runique start`",
    )
        .into_response()
}

async fn admin_stub_create_get() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération — lancez `runique start`",
    )
        .into_response()
}

async fn admin_stub_create_post() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération — lancez `runique start`",
    )
        .into_response()
}

async fn admin_stub_detail(_path: Path<i32>) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération — lancez `runique start`",
    )
        .into_response()
}

async fn admin_stub_edit(_path: Path<i32>) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération — lancez `runique start`",
    )
        .into_response()
}

async fn admin_stub_delete(_path: Path<i32>) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération — lancez `runique start`",
    )
        .into_response()
}
