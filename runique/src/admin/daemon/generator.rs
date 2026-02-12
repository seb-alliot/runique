use std::fmt::Write;
use std::fs;
use std::path::Path;

use crate::admin::daemon::parser::ResourceDef;

/// Génère 3 fichiers dans `src/admins/` : mod.rs, router.rs, handlers.rs
pub fn generate(resources: &[ResourceDef]) -> Result<(), String> {
    let admins_dir = Path::new("src/admins");

    // Créer ou vider le dossier admins/
    if admins_dir.exists() {
        fs::remove_dir_all(admins_dir)
            .map_err(|e| format!("Impossible de supprimer {}: {}", admins_dir.display(), e))?;
    }
    fs::create_dir_all(admins_dir)
        .map_err(|e| format!("Impossible de créer {}: {}", admins_dir.display(), e))?;

    // Générer les 3 fichiers
    generate_mod_file(resources, admins_dir)?;
    generate_router_file(resources, admins_dir)?;
    generate_handlers_file(resources, admins_dir)?;

    Ok(())
}

/// Génère src/admins/mod.rs
fn generate_mod_file(_resources: &[ResourceDef], dir: &Path) -> Result<(), String> {
    let mut out = String::new();

    let _ = writeln!(
        out,
        "// ═══════════════════════════════════════════════════════════════"
    );
    let _ = writeln!(
        out,
        "// AUTO-GÉNÉRÉ par Runique daemon — NE PAS MODIFIER MANUELLEMENT"
    );
    let _ = writeln!(
        out,
        "// ═══════════════════════════════════════════════════════════════"
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "pub mod handlers;");
    let _ = writeln!(out, "pub mod router;");
    let _ = writeln!(out);
    let _ = writeln!(out, "pub use router::admin;");

    fs::write(dir.join("mod.rs"), out).map_err(|e| format!("Impossible d'écrire mod.rs: {}", e))
}

/// Génère src/admins/router.rs
fn generate_router_file(resources: &[ResourceDef], dir: &Path) -> Result<(), String> {
    let mut out = String::new();

    // Header
    let _ = writeln!(
        out,
        "// ═══════════════════════════════════════════════════════════════"
    );
    let _ = writeln!(
        out,
        "// AUTO-GÉNÉRÉ par Runique daemon — NE PAS MODIFIER MANUELLEMENT"
    );
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

    // Imports
    let _ = writeln!(out, "use runique::prelude::*;");
    let _ = writeln!(out, "use runique::{{urlpatterns, view}};");
    let _ = writeln!(out, "use crate::admins::handlers;");
    let _ = writeln!(out);

    // Fonction admin()
    let _ = writeln!(out, "pub fn admin(prefix: &str) -> Router {{");
    let _ = writeln!(out, "    urlpatterns! {{");

    for r in resources {
        let key = &r.key;
        let _ = writeln!(out, "        &format!(\"{{}}/{}/list\", prefix) => view! {{ handlers::{}_list }}, name = \"{}_list\",",
            key, key, key);
        let _ = writeln!(out, "        &format!(\"{{}}/{}/create\", prefix) => view! {{ handlers::{}_create }}, name = \"{}_create\",",
            key, key, key);
        let _ = writeln!(out, "        &format!(\"{{}}/{}/{{{{id}}}}\", prefix) => view! {{ handlers::{}_detail }}, name = \"{}_detail\",",
            key, key, key);
        let _ = writeln!(out, "        &format!(\"{{}}/{}/{{{{id}}}}/edit\", prefix) => view! {{ handlers::{}_edit }}, name = \"{}_edit\",",
            key, key, key);
        let _ = writeln!(out, "        &format!(\"{{}}/{}/{{{{id}}}}/delete\", prefix) => view! {{ handlers::{}_delete }}, name = \"{}_delete\",",
            key, key, key);
    }

    let _ = writeln!(out, "    }}");
    let _ = writeln!(out, "}}");

    fs::write(dir.join("router.rs"), out)
        .map_err(|e| format!("Impossible d'écrire router.rs: {}", e))
}

/// Génère src/admins/handlers.rs
fn generate_handlers_file(resources: &[ResourceDef], dir: &Path) -> Result<(), String> {
    let mut out = String::new();

    // Header
    let _ = writeln!(
        out,
        "// ═══════════════════════════════════════════════════════════════"
    );
    let _ = writeln!(
        out,
        "// AUTO-GÉNÉRÉ par Runique daemon — NE PAS MODIFIER MANUELLEMENT"
    );
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
    let _ = writeln!(out, "#![allow(unused_imports, dead_code)]");
    let _ = writeln!(out);

    // Imports communs
    let _ = writeln!(out, "use runique::prelude::*;");
    let _ = writeln!(out, "use runique::utils::aliases::StrMap;");
    let _ = writeln!(out, "use std::sync::Arc;");
    let _ = writeln!(out);

    // Imports spécifiques par ressource
    for r in resources {
        let model_path = model_import_path(&r.model_type);
        let _ = writeln!(out, "use crate::{};", model_path);
        let _ = writeln!(out, "use crate::forms::{};", r.form_type);
    }
    let _ = writeln!(out);

    // Générer tous les handlers
    for r in resources {
        write_handler_list(&mut out, r)?;
        write_handler_create(&mut out, r)?;
        write_handler_edit(&mut out, r)?;
        write_handler_detail(&mut out, r)?;
        write_handler_delete(&mut out, r)?;
    }

    fs::write(dir.join("handlers.rs"), out)
        .map_err(|e| format!("Impossible d'écrire handlers.rs: {}", e))
}

/// Convertit "users::Model" → "models::users"
fn model_import_path(model_type: &str) -> String {
    let parts: Vec<&str> = model_type.split("::").collect();
    if parts.len() >= 2 {
        let module_parts = &parts[..parts.len() - 1];
        format!("models::{}", module_parts.join("::"))
    } else {
        format!("models::{}", model_type.to_lowercase())
    }
}

fn write_handler_list(out: &mut String, r: &ResourceDef) -> Result<(), String> {
    let key = &r.key;
    let model = &r.model_type;
    let form = &r.form_type;

    let _ = writeln!(out, "// ───────────── Handler {}_list ─────────────", key);
    let _ = writeln!(out, "pub async fn {}_list(", key);
    let _ = writeln!(out, "    mut req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, "    Prisme(mut form): Prisme<{}>", form);
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(out, "    if req.is_get() {{");
    let _ = writeln!(
        out,
        "        let entries = <{} as ModelTrait>::Entity::find()",
        model
    );
    let _ = writeln!(out, "            .all(&*req.engine.db)");
    let _ = writeln!(out, "            .await?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "        context_update!(req => {{");
    let _ = writeln!(out, "            \"resource_key\" => \"{}\",", key);
    let _ = writeln!(out, "            \"form_fields\" => &form,");
    let _ = writeln!(out, "            \"entries\" => &entries,");
    let _ = writeln!(out, "            \"total\" => entries.len()");
    let _ = writeln!(out, "        }});");
    let _ = writeln!(out, "        return req.render(\"admin/list\");");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    if req.is_post() {{");
    let _ = writeln!(out, "        if form.is_valid().await {{");
    let _ = writeln!(
        out,
        "            form.save(&req.engine.db).await.map_err(|err| {{"
    );
    let _ = writeln!(
        out,
        "                form.get_form_mut().database_error(&err);"
    );
    let _ = writeln!(out, "                AppError::from(err)");
    let _ = writeln!(out, "            }})?;");
    let _ = writeln!(
        out,
        "            success!(req.notices => \"Entrée créée avec succès !\");"
    );
    let _ = writeln!(out, "            return Ok(Redirect::to(&format!(\"/{{}}/{}/list\", admin.config.prefix)).into_response());", key);
    let _ = writeln!(out, "        }} else {{");
    let _ = writeln!(out, "            context_update!(req => {{");
    let _ = writeln!(out, "                \"resource_key\" => \"{}\",", key);
    let _ = writeln!(out, "                \"form_fields\" => &form,");
    let _ = writeln!(
        out,
        "                \"messages\" => flash_now!(error => \"Veuillez corriger les erreurs\")"
    );
    let _ = writeln!(out, "            }});");
    let _ = writeln!(out, "            return req.render(\"admin/list\");");
    let _ = writeln!(out, "        }}");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    req.render(\"admin/list\")");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);

    Ok(())
}

fn write_handler_create(out: &mut String, r: &ResourceDef) -> Result<(), String> {
    let key = &r.key;
    let form = &r.form_type;

    let _ = writeln!(out, "// ───────────── Handler {}_create ─────────────", key);
    let _ = writeln!(out, "pub async fn {}_create(", key);
    let _ = writeln!(out, "    mut req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, "    Prisme(mut form): Prisme<{}>", form);
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(out, "    if req.is_get() {{");
    let _ = writeln!(out, "        context_update!(req => {{");
    let _ = writeln!(out, "            \"resource_key\" => \"{}\",", key);
    let _ = writeln!(out, "            \"form_fields\" => &form,");
    let _ = writeln!(out, "            \"is_edit\" => false");
    let _ = writeln!(out, "        }});");
    let _ = writeln!(out, "        return req.render(\"admin/form\");");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    if req.is_post() {{");
    let _ = writeln!(out, "        if form.is_valid().await {{");
    let _ = writeln!(
        out,
        "            form.save(&req.engine.db).await.map_err(|err| {{"
    );
    let _ = writeln!(
        out,
        "                form.get_form_mut().database_error(&err);"
    );
    let _ = writeln!(out, "                AppError::from(err)");
    let _ = writeln!(out, "            }})?;");
    let _ = writeln!(
        out,
        "            success!(req.notices => \"Entrée créée avec succès !\");"
    );
    let _ = writeln!(out, "            return Ok(Redirect::to(&format!(\"/{{}}/{}/list\", admin.config.prefix)).into_response());", key);
    let _ = writeln!(out, "        }} else {{");
    let _ = writeln!(out, "            context_update!(req => {{");
    let _ = writeln!(out, "                \"resource_key\" => \"{}\",", key);
    let _ = writeln!(out, "                \"form_fields\" => &form,");
    let _ = writeln!(out, "                \"is_edit\" => false,");
    let _ = writeln!(
        out,
        "                \"messages\" => flash_now!(error => \"Veuillez corriger les erreurs\")"
    );
    let _ = writeln!(out, "            }});");
    let _ = writeln!(out, "            return req.render(\"admin/form\");");
    let _ = writeln!(out, "        }}");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    req.render(\"admin/form\")");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);

    Ok(())
}

fn write_handler_edit(out: &mut String, r: &ResourceDef) -> Result<(), String> {
    let key = &r.key;
    let model = &r.model_type;
    let form = &r.form_type;

    let _ = writeln!(out, "// ───────────── Handler {}_edit ─────────────", key);
    let _ = writeln!(out, "pub async fn {}_edit(", key);
    let _ = writeln!(out, "    mut req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, "    Path(id): Path<i32>,");
    let _ = writeln!(out, "    Prisme(mut form): Prisme<{}>", form);
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(
        out,
        "    let entry = <{} as ModelTrait>::Entity::find_by_id(id)",
        model
    );
    let _ = writeln!(out, "        .one(&*req.engine.db)");
    let _ = writeln!(out, "        .await?");
    let _ = writeln!(out, "        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found(\"Entry not found\"))))?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "    if req.is_get() {{");
    let _ = writeln!(
        out,
        "        // Convertir Model → StrMap via JSON pour pré-remplir le form"
    );
    let _ = writeln!(out, "        let entry_json = serde_json::to_value(&entry)");
    let _ = writeln!(
        out,
        "            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;"
    );
    let _ = writeln!(out, "        ");
    let _ = writeln!(out, "        let mut form_data = StrMap::new();");
    let _ = writeln!(out, "        if let Some(obj) = entry_json.as_object() {{");
    let _ = writeln!(out, "            for (k, v) in obj {{");
    let _ = writeln!(out, "                form_data.insert(k.clone(), v.to_string().trim_matches('\"').to_string());");
    let _ = writeln!(out, "            }}");
    let _ = writeln!(out, "        }}");
    let _ = writeln!(out, "        ");
    let _ = writeln!(
        out,
        "        // Remplir le form existant avec les données de l'entry"
    );
    let _ = writeln!(out, "        form.get_form_mut().fill(&form_data);");
    let _ = writeln!(out, "        ");
    let _ = writeln!(out, "        context_update!(req => {{");
    let _ = writeln!(out, "            \"resource_key\" => \"{}\",", key);
    let _ = writeln!(out, "            \"form_fields\" => &form,");
    let _ = writeln!(out, "            \"is_edit\" => true,");
    let _ = writeln!(out, "            \"object_id\" => id,");
    let _ = writeln!(out, "            \"entry\" => &entry");
    let _ = writeln!(out, "        }});");
    let _ = writeln!(out, "        return req.render(\"admin/form\");");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    if req.is_post() {{");
    let _ = writeln!(out, "        if form.is_valid().await {{");
    let _ = writeln!(
        out,
        "            form.save(&req.engine.db).await.map_err(|err| {{"
    );
    let _ = writeln!(
        out,
        "                form.get_form_mut().database_error(&err);"
    );
    let _ = writeln!(out, "                AppError::from(err)");
    let _ = writeln!(out, "            }})?;");
    let _ = writeln!(
        out,
        "            success!(req.notices => \"Entrée mise à jour avec succès !\");"
    );
    let _ = writeln!(out, "            return Ok(Redirect::to(&format!(\"/{{}}/{}/list\", admin.config.prefix)).into_response());", key);
    let _ = writeln!(out, "        }} else {{");
    let _ = writeln!(out, "            context_update!(req => {{");
    let _ = writeln!(out, "                \"resource_key\" => \"{}\",", key);
    let _ = writeln!(out, "                \"form_fields\" => &form,");
    let _ = writeln!(out, "                \"is_edit\" => true,");
    let _ = writeln!(out, "                \"object_id\" => id,");
    let _ = writeln!(out, "                \"entry\" => &entry,");
    let _ = writeln!(
        out,
        "                \"messages\" => flash_now!(error => \"Veuillez corriger les erreurs\")"
    );
    let _ = writeln!(out, "            }});");
    let _ = writeln!(out, "            return req.render(\"admin/form\");");
    let _ = writeln!(out, "        }}");
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(out, "    req.render(\"admin/form\")");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);

    Ok(())
}

fn write_handler_detail(out: &mut String, r: &ResourceDef) -> Result<(), String> {
    let key = &r.key;
    let model = &r.model_type;

    let _ = writeln!(out, "// ───────────── Handler {}_detail ─────────────", key);
    let _ = writeln!(out, "pub async fn {}_detail(", key);
    let _ = writeln!(out, "    mut req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, "    Path(id): Path<i32>");
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(
        out,
        "    let entry = <{} as ModelTrait>::Entity::find_by_id(id)",
        model
    );
    let _ = writeln!(out, "        .one(&*req.engine.db)");
    let _ = writeln!(out, "        .await?");
    let _ = writeln!(out, "        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found(\"Entry not found\"))))?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "    context_update!(req => {{");
    let _ = writeln!(out, "        \"resource_key\" => \"{}\",", key);
    let _ = writeln!(out, "        \"entry\" => &entry,");
    let _ = writeln!(out, "        \"object_id\" => id");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);
    let _ = writeln!(out, "    req.render(\"admin/detail\")");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);

    Ok(())
}

fn write_handler_delete(out: &mut String, r: &ResourceDef) -> Result<(), String> {
    let key = &r.key;
    let model = &r.model_type;

    let _ = writeln!(out, "// ───────────── Handler {}_delete ─────────────", key);
    let _ = writeln!(out, "pub async fn {}_delete(", key);
    let _ = writeln!(out, "    mut req: Request,");
    let _ = writeln!(out, "    Extension(admin): Extension<Arc<AdminState>>,");
    let _ = writeln!(out, "    Path(id): Path<i32>");
    let _ = writeln!(out, ") -> AppResult<Response> {{");
    let _ = writeln!(out, "    if req.is_post() {{");
    let _ = writeln!(
        out,
        "        let entry = <{} as ModelTrait>::Entity::find_by_id(id)",
        model
    );
    let _ = writeln!(out, "            .one(&*req.engine.db)");
    let _ = writeln!(out, "            .await?");
    let _ = writeln!(out, "            .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found(\"Entry not found\"))))?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "        entry.delete(&*req.engine.db).await?;");
    let _ = writeln!(
        out,
        "        success!(req.notices => \"Entrée supprimée avec succès !\");"
    );
    let _ = writeln!(out, "        return Ok(Redirect::to(&format!(\"/{{}}/{}/list\", admin.config.prefix)).into_response());", key);
    let _ = writeln!(out, "    }}");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "    let entry = <{} as ModelTrait>::Entity::find_by_id(id)",
        model
    );
    let _ = writeln!(out, "        .one(&*req.engine.db)");
    let _ = writeln!(out, "        .await?");
    let _ = writeln!(out, "        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found(\"Entry not found\"))))?;");
    let _ = writeln!(out);
    let _ = writeln!(out, "    context_update!(req => {{");
    let _ = writeln!(out, "        \"resource_key\" => \"{}\",", key);
    let _ = writeln!(out, "        \"entry\" => &entry,");
    let _ = writeln!(out, "        \"object_id\" => id");
    let _ = writeln!(out, "    }});");
    let _ = writeln!(out);
    let _ = writeln!(out, "    req.render(\"admin/delete_confirm\")");
    let _ = writeln!(out, "}}");
    let _ = writeln!(out);

    Ok(())
}
