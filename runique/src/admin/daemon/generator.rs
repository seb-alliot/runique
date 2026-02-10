// ═══════════════════════════════════════════════════════════════
// Generator — Génération de target/runique/admin/generated.rs
// ═══════════════════════════════════════════════════════════════
//
// Prend les ResourceDef parsés et génère du code Rust valide :
//   - admin_registry() → construit l'AdminRegistry
//   - handlers CRUD type-safe par ressource
//   - admin_router() → Router Axum câblé
//
// Stratégie : génération par template string plutôt que quote,
// car la sortie est un fichier .rs standalone (pas un proc-macro).
// ═══════════════════════════════════════════════════════════════

use chrono::Local;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::admin::daemon::parser::ResourceDef;

/// Génère target/runique/admin/generated.rs depuis les ResourceDef
pub fn generate(resources: &[ResourceDef], output_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Impossible de créer {}: {}", output_dir.display(), e))?;

    let code = generate_code(resources)?;

    let output_path = output_dir.join("generated.rs");
    fs::write(&output_path, &code)
        .map_err(|e| format!("Impossible d'écrire {}: {}", output_path.display(), e))?;

    Ok(())
}

/// Génère le code Rust complet
fn generate_code(resources: &[ResourceDef]) -> Result<String, String> {
    let mut out = String::new();

    write_header(&mut out, resources);
    write_imports(&mut out, resources);
    write_registry_fn(&mut out, resources);
    write_handlers(&mut out, resources);
    write_router_fn(&mut out, resources);

    Ok(out)
}

// ───────────────────────────────────────────────
// Header
// ───────────────────────────────────────────────

fn write_header(out: &mut String, resources: &[ResourceDef]) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let _ = writeln!(
        out,
        "// ═══════════════════════════════════════════════════════════════"
    );
    let _ = writeln!(
        out,
        "// AUTO-GÉNÉRÉ par Runique daemon — NE PAS MODIFIER MANUELLEMENT"
    );
    let _ = writeln!(out, "// Source  : src/admin.rs");
    let _ = writeln!(out, "// Généré  : {}", now);
    let _ = writeln!(
        out,
        "// Ressources : {}",
        resources
            .iter()
            .map(|r| r.key.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(
        out,
        "// ═══════════════════════════════════════════════════════════════"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "#![allow(unused_imports, dead_code)]");
    let _ = writeln!(out);
}

// ───────────────────────────────────────────────
// Imports
// ───────────────────────────────────────────────

fn write_imports(out: &mut String, resources: &[ResourceDef]) {
    let _ = writeln!(out, "use axum::{{");
    let _ = writeln!(out, "    extract::{{Path, Extension}},");
    let _ = writeln!(out, "    http::StatusCode,");
    let _ = writeln!(out, "    response::{{Html, IntoResponse, Response}},");
    let _ = writeln!(out, "    routing::{{get, post}},");
    let _ = writeln!(out, "    Router,");
    let _ = writeln!(out, "}};");
    let _ = writeln!(
        out,
        "use runique::admin::{{AdminRegistry, AdminResource, ResourcePermissions}};"
    );
    let _ = writeln!(out, "use runique::middleware::auth::CurrentUser;");
    let _ = writeln!(out, "use runique::utils::aliases::ADb;");
    let _ = writeln!(out);

    // Imports des models et forms
    for r in resources {
        // Transforme "users::Model" → "crate::models::users"
        // et "RegisterForm" → "crate::forms::RegisterForm"
        let model_import = model_import_path(&r.model_type);
        let _ = writeln!(out, "use crate::{};", model_import);
    }
    let _ = writeln!(out);
}

/// Transforme "users::Model" → "models::users" (module parent du Model)
fn model_import_path(model_type: &str) -> String {
    let parts: Vec<&str> = model_type.split("::").collect();
    if parts.len() >= 2 {
        // "users::Model" → "models::users" (on importe le module, pas le type)
        let module_parts = &parts[..parts.len() - 1];
        format!("models::{}", module_parts.join("::"))
    } else {
        format!("models::{}", model_type.to_lowercase())
    }
}

// ───────────────────────────────────────────────
// Fonction admin_registry()
// ───────────────────────────────────────────────

fn write_registry_fn(out: &mut String, resources: &[ResourceDef]) {
    let _ = writeln!(
        out,
        "/// Construit l'AdminRegistry avec toutes les ressources déclarées"
    );
    let _ = writeln!(out, "pub fn admin_registry() -> AdminRegistry {{");
    let _ = writeln!(out, "    let mut registry = AdminRegistry::new();");
    let _ = writeln!(out);

    for r in resources {
        let permissions: Vec<String> = r
            .permissions
            .iter()
            .map(|p| format!("    \"{}\".to_string()", p))
            .collect();

        let _ = writeln!(out, "    // Ressource: {}", r.key);
        let _ = writeln!(out, "    registry.register(AdminResource::new(");
        let _ = writeln!(out, "        \"{}\",", r.key);
        let _ = writeln!(out, "        \"{}\",", r.model_type);
        let _ = writeln!(out, "        \"{}\",", r.form_type);
        let _ = writeln!(out, "        \"{}\",", r.title);
        let _ = writeln!(out, "        vec![");
        let _ = writeln!(out, "{}", permissions.join(",\n"));
        let _ = writeln!(out, "        ],");
        let _ = writeln!(out, "    ));");
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "    registry");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
}

// ───────────────────────────────────────────────
// Handlers CRUD par ressource
// ───────────────────────────────────────────────

fn write_handlers(out: &mut String, resources: &[ResourceDef]) {
    for r in resources {
        let key = &r.key;
        let model = &r.model_type; // "users::Model"
        let form = &r.form_type; // "RegisterForm"
        let perms: Vec<String> = r.permissions.iter().map(|p| format!("\"{}\"", p)).collect();
        let perms_str = perms.join(", ");

        let _ = writeln!(out, "// ─────────────────────────────────────────────");
        let _ = writeln!(out, "// Handlers: {} ({} → {})", key, model, form);
        let _ = writeln!(out, "// ─────────────────────────────────────────────");
        let _ = writeln!(out);

        // LIST
        let _ = writeln!(out, "pub async fn admin_{key}_list(");
        let _ = writeln!(out, "    Extension(user): Extension<CurrentUser>,");
        let _ = writeln!(out, "    Extension(db): Extension<ADb>,");
        let _ = writeln!(out, ") -> Response {{");
        let _ = writeln!(out, "    if !user.can_admin(&[{perms_str}]) {{");
        let _ = writeln!(
            out,
            "        return (StatusCode::FORBIDDEN, \"Accès refusé\").into_response();"
        );
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "    // TODO: récupérer les entrées depuis la DB");
        let _ = writeln!(
            out,
            "    Html(format!(\"<h1>Liste {key}</h1>\")).into_response()"
        );
        let _ = writeln!(out, "}}");
        let _ = writeln!(out);

        // CREATE GET
        let _ = writeln!(out, "pub async fn admin_{key}_create_get(");
        let _ = writeln!(out, "    Extension(user): Extension<CurrentUser>,");
        let _ = writeln!(out, ") -> Response {{");
        let _ = writeln!(out, "    if !user.can_admin(&[{perms_str}]) {{");
        let _ = writeln!(
            out,
            "        return (StatusCode::FORBIDDEN, \"Accès refusé\").into_response();"
        );
        let _ = writeln!(out, "    }}");
        let _ = writeln!(
            out,
            "    Html(format!(\"<h1>Créer {key}</h1>\")).into_response()"
        );
        let _ = writeln!(out, "}}");
        let _ = writeln!(out);

        // CREATE POST
        let _ = writeln!(out, "pub async fn admin_{key}_create_post(");
        let _ = writeln!(out, "    Extension(user): Extension<CurrentUser>,");
        let _ = writeln!(out, "    Extension(db): Extension<ADb>,");
        let _ = writeln!(out, ") -> Response {{");
        let _ = writeln!(out, "    if !user.can_admin(&[{perms_str}]) {{");
        let _ = writeln!(
            out,
            "        return (StatusCode::FORBIDDEN, \"Accès refusé\").into_response();"
        );
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "    // TODO: traiter le formulaire {form}");
        let _ = writeln!(out, "    (StatusCode::CREATED, \"Créé\").into_response()");
        let _ = writeln!(out, "}}");
        let _ = writeln!(out);

        // DETAIL GET
        let _ = writeln!(out, "pub async fn admin_{key}_detail(");
        let _ = writeln!(out, "    Extension(user): Extension<CurrentUser>,");
        let _ = writeln!(out, "    Extension(db): Extension<ADb>,");
        let _ = writeln!(out, "    Path(id): Path<i32>,");
        let _ = writeln!(out, ") -> Response {{");
        let _ = writeln!(out, "    if !user.can_admin(&[{perms_str}]) {{");
        let _ = writeln!(
            out,
            "        return (StatusCode::FORBIDDEN, \"Accès refusé\").into_response();"
        );
        let _ = writeln!(out, "    }}");
        let _ = writeln!(
            out,
            "    Html(format!(\"<h1>{key} #{{id}}</h1>\")).into_response()"
        );
        let _ = writeln!(out, "}}");
        let _ = writeln!(out);

        // EDIT POST
        let _ = writeln!(out, "pub async fn admin_{key}_edit(");
        let _ = writeln!(out, "    Extension(user): Extension<CurrentUser>,");
        let _ = writeln!(out, "    Extension(db): Extension<ADb>,");
        let _ = writeln!(out, "    Path(id): Path<i32>,");
        let _ = writeln!(out, ") -> Response {{");
        let _ = writeln!(out, "    if !user.can_admin(&[{perms_str}]) {{");
        let _ = writeln!(
            out,
            "        return (StatusCode::FORBIDDEN, \"Accès refusé\").into_response();"
        );
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "    // TODO: traiter l'édition {form}");
        let _ = writeln!(
            out,
            "    Html(format!(\"<h1>Éditer {key} #{{id}}</h1>\")).into_response()"
        );
        let _ = writeln!(out, "}}");
        let _ = writeln!(out);

        // DELETE POST
        let _ = writeln!(out, "pub async fn admin_{key}_delete(");
        let _ = writeln!(out, "    Extension(user): Extension<CurrentUser>,");
        let _ = writeln!(out, "    Extension(db): Extension<ADb>,");
        let _ = writeln!(out, "    Path(id): Path<i32>,");
        let _ = writeln!(out, ") -> Response {{");
        let _ = writeln!(out, "    if !user.can_admin(&[{perms_str}]) {{");
        let _ = writeln!(
            out,
            "        return (StatusCode::FORBIDDEN, \"Accès refusé\").into_response();"
        );
        let _ = writeln!(out, "    }}");
        let _ = writeln!(out, "    // TODO: supprimer l'entrée #{key} id={{id}}");
        let _ = writeln!(out, "    (StatusCode::OK, \"Supprimé\").into_response()");
        let _ = writeln!(out, "}}");
        let _ = writeln!(out);
    }
}

// ───────────────────────────────────────────────
// Router Axum câblé
// ───────────────────────────────────────────────

fn write_router_fn(out: &mut String, resources: &[ResourceDef]) {
    let _ = writeln!(
        out,
        "/// Router admin généré — câble tous les handlers type-safe"
    );
    let _ = writeln!(
        out,
        "pub fn generated_admin_router(prefix: &str) -> Router {{"
    );
    let _ = writeln!(out, "    let mut router = Router::new();");
    let _ = writeln!(out);

    for r in resources {
        let key = &r.key;
        let _ = writeln!(out, "    // {key}");
        let _ = writeln!(out, "    router = router");
        let _ = writeln!(
            out,
            "        .route(\"/{key}/list\", get(admin_{key}_list))"
        );
        let _ = writeln!(out, "        .route(\"/{key}/create\", get(admin_{key}_create_get).post(admin_{key}_create_post))");
        let _ = writeln!(
            out,
            "        .route(\"/{key}/{{id}}\", get(admin_{key}_detail).post(admin_{key}_edit))"
        );
        let _ = writeln!(
            out,
            "        .route(\"/{key}/{{id}}/delete\", post(admin_{key}_delete));"
        );
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "    Router::new().nest(prefix, router)");
    let _ = writeln!(out, "}}");
}
