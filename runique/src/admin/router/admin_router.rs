// ═══════════════════════════════════════════════════════════════
// Admin Router — Génération des routes CRUD
// ═══════════════════════════════════════════════════════════════
//
// Construit le Router Axum pour toutes les ressources enregistrées
// dans l'AdminRegistry.
//
// Routes générées pour chaque ressource (ex: "users") :
//   GET  /admin/users/list         → liste paginée
//   GET  /admin/users/create       → formulaire création
//   POST /admin/users/create       → traitement création
//   GET  /admin/users/:id          → détail / formulaire édition
//   POST /admin/users/:id          → traitement édition
//   POST /admin/users/:id/delete   → suppression (POST pour CSRF)
//
// Note : Les handlers réels sont dans target/runique/admin/generated.rs
// (généré par le daemon). Ce module fournit les handlers par défaut
// (stubs) et le builder de routes.
// ═══════════════════════════════════════════════════════════════

use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};

use crate::admin::config::AdminConfig;
use crate::admin::registry::AdminRegistry;

// ───────────────────────────────────────────────
// Builder de routes admin
// ───────────────────────────────────────────────

/// Construit le Router admin complet depuis l'AdminRegistry
///
/// Génère automatiquement les routes CRUD pour chaque ressource.
/// Les handlers sont des stubs jusqu'à ce que le daemon génère
/// les handlers type-safe dans `target/runique/admin/generated.rs`.
pub fn build_admin_router(registry: &AdminRegistry, config: &AdminConfig) -> Router {
    let mut router = Router::new();

    // Route racine admin → dashboard
    router = router.route("/", get(admin_dashboard));

    // Route login admin
    router = router.route("/login", get(admin_login_get).post(admin_login_post));

    // Routes CRUD pour chaque ressource enregistrée
    for resource in &registry.resources {
        let key = resource.key;

        router = router
            // Liste
            .route(&format!("/{}/list", key), get(admin_stub_list))
            // Création
            .route(
                &format!("/{}/create", key),
                get(admin_stub_create_get).post(admin_stub_create_post),
            )
            // Détail / édition
            .route(
                &format!("/{key}/:id"),
                get(admin_stub_detail).post(admin_stub_edit),
            )
            // Suppression
            .route(&format!("/{key}/:id/delete"), post(admin_stub_delete));
    }

    // Préfixe global défini dans AdminConfig
    Router::new().nest(&config.prefix, router)
}

// ───────────────────────────────────────────────
// Handlers stubs (remplacés par le daemon)
// ───────────────────────────────────────────────
//
// Ces handlers sont des placeholders fonctionnels.
// Le daemon génère les vrais handlers type-safe dans
// target/runique/admin/generated.rs.

async fn admin_dashboard() -> Response {
    Html("<h1>Admin Dashboard</h1><p>Runique AdminPanel — en cours de génération</p>")
        .into_response()
}

async fn admin_login_get() -> Response {
    Html("<h1>Admin Login</h1>").into_response()
}

async fn admin_login_post() -> Response {
    (StatusCode::METHOD_NOT_ALLOWED, "Non implémenté").into_response()
}

async fn admin_stub_list() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération",
    )
        .into_response()
}

async fn admin_stub_create_get() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération",
    )
        .into_response()
}

async fn admin_stub_create_post() -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération",
    )
        .into_response()
}

async fn admin_stub_detail(_path: Path<i32>) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération",
    )
        .into_response()
}

async fn admin_stub_edit(_path: Path<i32>) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération",
    )
        .into_response()
}

async fn admin_stub_delete(_path: Path<i32>) -> Response {
    (
        StatusCode::NOT_IMPLEMENTED,
        "Handler en cours de génération",
    )
        .into_response()
}
