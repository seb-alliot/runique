// Prend les ResourceDef parsés et génère du code Rust valide :
//   - admin_registry() → construit l'AdminRegistry
//   - handlers CRUD type-safe par ressource
//   - admin_router() → Router Axum câblé
//
// Stratégie : génération par template string plutôt que quote,
// car la sortie est un fichier .rs standalone (pas un proc-macro).

use chrono::Local;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::admin::daemon::parser::ResourceDef;

/// Génère target/runique/admin/generated.rs depuis les ResourceDef
pub fn generate(resources: &[ResourceDef], output_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Unable to create {}: {}", output_dir.display(), e))?;

    let code = generate_code(resources)?;

    let output_path = output_dir.join("generated.rs");
    fs::write(&output_path, &code)
        .map_err(|e| format!("Unable to write {}: {}", output_path.display(), e))?;

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

fn write_imports(out: &mut String, resources: &[ResourceDef]) {
    // Prelude contient tout : Router, Request, Response, AppResult, AppError, Redirect, etc.
    let _ = writeln!(out, "use runique::prelude::*;");
    let _ = writeln!(
        out,
        "use runique::admin::{{AdminRegistry, AdminResource, AdminState}};"
    );
    let _ = writeln!(out, "use runique::errors::error::ErrorContext;");
    let _ = writeln!(out, "use serde_json::{{json, Value}};");
    let _ = writeln!(out, "use std::sync::Arc;");
    let _ = writeln!(out);

    // Imports des models et forms
    for r in resources {
        let model_import = model_import_path(&r.model_type);
        let _ = writeln!(out, "use crate::{};", model_import);
        let _ = writeln!(out, "use crate::forms::{};", r.form_type);
    }
    let _ = writeln!(out);
}

/// Transforme "users::Model" → "models::users" (module parent du Model)
fn model_import_path(model_type: &str) -> String {
    let parts: Vec<&str> = model_type.split("::").collect();
    if parts.len() >= 2 {
        // "users::Model" → "models::users"
        let module_parts = &parts[..parts.len() - 1];
        format!("models::{}", module_parts.join("::"))
    } else {
        format!("models::{}", model_type.to_lowercase())
    }
}

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

fn write_handlers(out: &mut String, resources: &[ResourceDef]) {
    for r in resources {
        let key = &r.key;
        let model = &r.model_type;
        let form = &r.form_type;
        let perms: Vec<String> = r.permissions.iter().map(|p| format!("\"{}\"", p)).collect();
        let perms_str = perms.join(", ");

        let _ = writeln!(out, "// ─────────────────────────────────────────────");
        let _ = writeln!(out, "// Handlers: {} ({} → {})", key, model, form);
        let _ = writeln!(out, "// ─────────────────────────────────────────────");
        let _ = writeln!(out);

        write_list_handler(out, key, model, &perms_str);
        write_create_get_handler(out, key, form, &perms_str);
        write_create_post_handler(out, key, model, form, &perms_str);
        write_detail_handler(out, key, model, &perms_str);
        write_edit_handler(out, key, model, form, &perms_str);
        write_delete_handler(out, key, model, &perms_str);
    }
}

fn write_list_handler(out: &mut String, key: &str, model: &str, _perms_str: &str) {
    let _ = writeln!(out, "pub async fn admin_{key}_list(");
    let _ = writeln!(out, "    mut req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Trouver la resource");
    let _ = writeln!(out, "    let resource = admin.registry.resources.iter()");
    let _ = writeln!(out, "        .find(|r| r.key == \"{key}\")");
    let _ = writeln!(out, "        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found(\"Resource not found\"))))?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Récupérer toutes les entrées");
    let _ = writeln!(
        out,
        "    let entries = <{model} as runique::prelude::ModelTrait>::Entity::find()"
    );
    let _ = writeln!(out, "        .all(&*req.engine.db)");
    let _ = writeln!(out, "        .await?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Convertir en JSON");
    let _ = writeln!(out, "    let rows: Vec<Value> = entries.iter()");
    let _ = writeln!(
        out,
        "        .map(|entry| serde_json::to_value(entry).unwrap_or(json!({{}})))"
    );
    let _ = writeln!(out, "        .collect();");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Extraire les colonnes");
    let _ = writeln!(
        out,
        "    let columns = if let Some(first) = rows.first() {{"
    );
    let _ = writeln!(out, "        first.as_object()");
    let _ = writeln!(out, "            .map(|obj| obj.keys().cloned().collect())");
    let _ = writeln!(out, "            .unwrap_or_default()");
    let _ = writeln!(out, "    }} else {{");
    let _ = writeln!(out, "        Vec::new()");
    let _ = writeln!(out, "    }};");
    let _ = writeln!(out);
    let _ = writeln!(out, "    context_update!(req => {{");
    let _ = writeln!(out, "        \"resource\" => resource,");
    let _ = writeln!(out, "        \"rows\" => rows,");
    let _ = writeln!(out, "        \"columns\" => columns,");
    let _ = writeln!(out, "        \"total\" => entries.len(),");
    let _ = writeln!(out, "        \"current_page\" => 1,");
    let _ = writeln!(out, "        \"total_pages\" => 1,");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);
    let _ = writeln!(out, "    req.render(\"admin/list\")");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
}

fn write_create_get_handler(out: &mut String, key: &str, form: &str, _perms_str: &str) {
    let _ = writeln!(out, "pub async fn admin_{key}_create_get(");
    let _ = writeln!(out, "    mut req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, "    Prisme(mut form): Prisme<{form}>,");
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Trouver la ressource");
    let _ = writeln!(out, "    let resource = admin.registry.resources.iter()");
    let _ = writeln!(out, "        .find(|r| r.key == \"{key}\")");
    let _ = writeln!(out, "        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found(\"Resource not found\"))))?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Sérialiser le form en JSON pour Tera");
    let _ = writeln!(out, "    let form_json = serde_json::to_value(&form.form)");
    let _ = writeln!(
        out,
        "        .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;"
    );
    let _ = writeln!(
        out,
        "    let form_fields = form_json.get(\"fields\").cloned().unwrap_or(json!({{}}));"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "    context_update!(req => {{");
    let _ = writeln!(out, "        \"resource\" => resource,");
    let _ = writeln!(out, "        \"form_fields\" => form_fields,");
    let _ = writeln!(out, "        \"is_edit\" => false,");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);
    let _ = writeln!(out, "    req.render(\"admin/form\")");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
}

fn write_create_post_handler(
    out: &mut String,
    key: &str,
    _model: &str,
    form: &str,
    _perms_str: &str,
) {
    let _ = writeln!(out, "pub async fn admin_{key}_create_post(");
    let _ = writeln!(out, "    mut req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, "    Prisme(mut form): Prisme<{form}>,");
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(out);
    let _ = writeln!(out, "    if form.is_valid().await {{");
    let _ = writeln!(
        out,
        "        form.save(&*req.engine.db).await.map_err(|err| {{"
    );
    let _ = writeln!(out, "            form.get_form_mut().database_error(&err);");
    let _ = writeln!(out, "            AppError::from(err)");
    let _ = writeln!(out, "        }})?;");
    let _ = writeln!(
        out,
        "        success!(req.notices => \"Entrée créée avec succès !\");"
    );
    let _ = writeln!(out, "        return Ok(Redirect::to(&format!(\"/{{}}/{key}/list\", admin.config.prefix)).into_response());");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Validation échouée → réafficher le formulaire");
    let _ = writeln!(
        out,
        "    let form_json = serde_json::to_value(&form.form)
        .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;"
    );
    let _ = writeln!(
        out,
        "    let form_fields = form_json.get(\"fields\").cloned().unwrap_or(json!({{}}));"
    );
    let _ = writeln!(out, "    context_update!(req => {{");
    let _ = writeln!(out, "        \"form_fields\" => form_fields,");
    let _ = writeln!(out, "        \"is_edit\" => false,");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);
    let _ = writeln!(out, "    req.render(\"admin/form\")");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
}

fn write_detail_handler(out: &mut String, key: &str, model: &str, _perms_str: &str) {
    let _ = writeln!(out, "pub async fn admin_{key}_detail(");
    let _ = writeln!(out, "    mut req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, "    Path(id): Path<i32>,");
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Trouver la resource");
    let _ = writeln!(out, "    let resource = admin.registry.resources.iter()");
    let _ = writeln!(out, "        .find(|r| r.key == \"{key}\")");
    let _ = writeln!(out, "        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found(\"Resource not found\"))))?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Récupérer l'entrée par ID");
    let _ = writeln!(
        out,
        "    let entry = <{model} as runique::prelude::ModelTrait>::Entity::find_by_id(id)"
    );
    let _ = writeln!(out, "        .one(&*req.engine.db)");
    let _ = writeln!(out, "        .await?");
    let _ = writeln!(out, "        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found(\"Entry not found\"))))?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Convertir en JSON");
    let _ = writeln!(out, "    let object = serde_json::to_value(&entry)");
    let _ = writeln!(
        out,
        "        .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "    context_update!(req => {{");
    let _ = writeln!(out, "        \"resource\" => resource,");
    let _ = writeln!(out, "        \"object\" => object,");
    let _ = writeln!(out, "        \"object_id\" => id,");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);
    let _ = writeln!(out, "    req.render(\"admin/detail\")");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
}

fn write_edit_handler(out: &mut String, key: &str, model: &str, form: &str, _perms_str: &str) {
    let _ = writeln!(out, "pub async fn admin_{key}_edit(");
    let _ = writeln!(out, "    mut req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, "    Path(id): Path<i32>,");
    let _ = writeln!(out, "    Prisme(mut form): Prisme<{form}>,");
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Récupérer l'entrée existante");
    let _ = writeln!(
        out,
        "    let mut entry = <{model} as runique::prelude::ModelTrait>::Entity::find_by_id(id)"
    );
    let _ = writeln!(out, "        .one(&*req.engine.db)");
    let _ = writeln!(out, "        .await?");
    let _ = writeln!(out, "        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found(\"Entry not found\"))))?;");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "    // Pré-remplir le form avec les valeurs actuelles si pas déjà rempli"
    );
    let _ = writeln!(out, "    if form.form.fields.is_empty() {{");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    if form.is_valid().await {{");
    let _ = writeln!(out, "        // Mettre à jour l'entrée en DB");
    let _ = writeln!(
        out,
        "        form.save(&*req.engine.db).await.map_err(|err| {{"
    );
    let _ = writeln!(out, "            form.get_form_mut().database_error(&err);");
    let _ = writeln!(out, "            AppError::from(err)");
    let _ = writeln!(out, "        }})?;");
    let _ = writeln!(
        out,
        "        success!(req.notices => \"Entrée mise à jour avec succès !\");"
    );
    let _ = writeln!(out, "        return Ok(Redirect::to(&format!(\"/{{}}/{key}/list\", admin.config.prefix)).into_response());");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Validation échouée → réafficher le formulaire");
    let _ = writeln!(
        out,
        "    let form_json = serde_json::to_value(&form.form)
    .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;"
    );
    let _ = writeln!(
        out,
        "    let form_fields = form_json.get(\"fields\").cloned().unwrap_or(json!({{}}));"
    );
    let _ = writeln!(out, "    context_update!(req => {{");
    let _ = writeln!(out, "        \"form_fields\" => form_fields,");
    let _ = writeln!(out, "        \"is_edit\" => true,");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);
    let _ = writeln!(out, "    req.render(\"admin/form\")");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
}

fn write_delete_handler(out: &mut String, key: &str, model: &str, _perms_str: &str) {
    let _ = writeln!(out, "pub async fn admin_{key}_delete(");
    let _ = writeln!(out, "    req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, "    Path(id): Path<i32>,");
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Récupérer l'entrée");
    let _ = writeln!(
        out,
        "    let entry = <{model} as runique::prelude::ModelTrait>::Entity::find_by_id(id)"
    );
    let _ = writeln!(out, "        .one(&*req.engine.db)");
    let _ = writeln!(out, "        .await?");
    let _ = writeln!(out, "        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found(\"Entry not found\"))))?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "    // Supprimer");
    let _ = writeln!(out, "    entry.delete(&*req.engine.db).await?;");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "    Ok(Redirect::to(&format!(\"/{{}}/{key}/list\", admin.config.prefix)).into_response())"
    );
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);
}

fn write_router_fn(out: &mut String, resources: &[ResourceDef]) {
    let _ = writeln!(
        out,
        "/// Enregistre les routes CRUD générées sur le router donné"
    );
    let _ = writeln!(
        out,
        "pub fn register_crud_routes(router: Router, prefix: &str) -> Router {{"
    );
    let _ = writeln!(out, "    let mut r = router;");
    let _ = writeln!(out);

    for r in resources {
        let key = &r.key;
        let _ = writeln!(out, "    // Routes pour {key}");
        let _ = writeln!(out, "    r = r");
        let _ = writeln!(
            out,
            "        .route(&format!(\"{{}}/{key}/list\", prefix), get(admin_{key}_list))"
        );
        let _ = writeln!(out, "        .route(&format!(\"{{}}/{key}/create\", prefix), get(admin_{key}_create_get).post(admin_{key}_create_post))");
        let _ = writeln!(
            out,
            "        .route(&format!(\"{{}}/{key}/{{{{id}}}}\", prefix), get(admin_{key}_detail).post(admin_{key}_edit))"
        );
        let _ = writeln!(
            out,
            "        .route(&format!(\"{{}}/{key}/{{{{id}}}}/delete\", prefix), post(admin_{key}_delete));"
        );
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "    r");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);

    // Générer aussi une fonction qui construit le router complet
    let _ = writeln!(
        out,
        "/// Construit un Router admin complet avec toutes les routes CRUD"
    );
    let _ = writeln!(
        out,
        "pub fn build_generated_router(prefix: &str, admin_state: Arc<AdminState>) -> Router {{"
    );
    let _ = writeln!(out, "    // Routes CRUD générées");
    let _ = writeln!(
        out,
        "    let crud_router = register_crud_routes(Router::new(), prefix);"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "    crud_router.layer(Extension(admin_state))");
    let _ = writeln!(out, "}}");
}
