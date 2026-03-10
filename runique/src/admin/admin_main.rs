// ═══════════════════════════════════════════════════════════════
// admin_main — Handler générique central de l'interface admin
//
// Routes :
//   /admin/{resource}/{action}          → admin_get / admin_post
//   /admin/{resource}/{id}/{action}     → admin_get_id / admin_post_id
// ═══════════════════════════════════════════════════════════════

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{FromRequest, Path},
    http::{Request as HttpRequest, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use serde_json::Value;

use crate::admin::config::AdminConfig;
use crate::admin::trad::insert_admin_messages;
use crate::admin::AdminRegistry;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::flash_now;
use crate::forms::prisme::aegis;
use crate::utils::aliases::{ARuniqueConfig, AppResult, StrMap};
use crate::utils::trad::{current_lang, t};

// ─── Extracteur AdminBody ─────────────────────────────────────
//
// Encapsule aegis pour gérer automatiquement multipart/form-data
// ET application/x-www-form-urlencoded dans les handlers admin POST.
// Remplace axum::extract::Form<StrMap> qui refusait le multipart.

pub struct AdminBody(StrMap);

impl<S: Send + Sync> FromRequest<S> for AdminBody {
    type Rejection = Response;

    async fn from_request(req: HttpRequest<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let config = req
            .extensions()
            .get::<ARuniqueConfig>()
            .cloned()
            .ok_or_else(|| {
                (StatusCode::INTERNAL_SERVER_ERROR, "config manquante").into_response()
            })?;

        let content_type = req
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let parsed = aegis(req, state, config, &content_type).await?;

        // StrVecMap → StrMap (multi-values jointes par virgule, comme Prisme)
        let body = parsed.into_iter().map(|(k, v)| (k, v.join(","))).collect();
        Ok(AdminBody(body))
    }
}

// ─── État partagé ────────────────────────────────────────────

#[derive(Clone)]
pub struct PrototypeAdminState {
    pub registry: Arc<AdminRegistry>,
    pub config: Arc<AdminConfig>,
}

// ─── Points d'entrée Axum ────────────────────────────────────

/// GET /admin/{resource}/{action}  (list, create)
pub async fn admin_get(
    mut req: Request,
    Path((resource_key, action)): Path<(String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry);

    match action.as_str() {
        "list" => handle_list(&mut req, entry, &state).await,
        "create" => handle_create_get(&mut req, entry, &state).await,
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Action inconnue",
        )))),
    }
}

/// POST /admin/{resource}/{action}  (create)
#[allow(private_interfaces)]
pub async fn admin_post(
    mut req: Request,
    Path((resource_key, action)): Path<(String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    AdminBody(body): AdminBody,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry);
    req.context.insert("lang", &current_lang().code());

    match action.as_str() {
        "create" => handle_create_post(&mut req, entry, body, &state).await,
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Action inconnue",
        )))),
    }
}

/// GET /admin/{resource}/{id}/{action}  (detail, edit, delete)
pub async fn admin_get_id(
    mut req: Request,
    Path((resource_key, id, action)): Path<(String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry);
    req.context.insert("lang", &current_lang().code());
    match action.as_str() {
        "detail" => handle_detail(&mut req, entry, id, &state).await,
        "edit" => handle_edit_get(&mut req, entry, id, &state).await,
        "delete" => handle_delete_get(&mut req, entry, id, &state).await,
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Action inconnue",
        )))),
    }
}

/// POST /admin/{resource}/{id}/{action}  (edit, delete)
#[allow(private_interfaces)]
pub async fn admin_post_id(
    mut req: Request,
    Path((resource_key, id, action)): Path<(String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    AdminBody(body): AdminBody,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry);
    req.context.insert("lang", &current_lang().code());

    match action.as_str() {
        "edit" => handle_edit_post(&mut req, entry, id, body, &state).await,
        "delete" => handle_delete_post(&mut req, entry, id, &state).await,
        _ => Err(Box::new(AppError::new(ErrorContext::not_found(
            "Action inconnue",
        )))),
    }
}

// ─── Fonctions internes ──────────────────────────────────────

fn inject_context(
    req: &mut Request,
    state: &PrototypeAdminState,
    entry: &crate::admin::ResourceEntry,
) {
    insert_admin_messages(&mut req.context, "list");
    insert_admin_messages(&mut req.context, "create");
    insert_admin_messages(&mut req.context, "edit");
    insert_admin_messages(&mut req.context, "detail");
    insert_admin_messages(&mut req.context, "delete");
    insert_admin_messages(&mut req.context, "base");

    req.context.insert("site_title", &state.config.site_title);
    req.context.insert("resource_key", entry.meta.key);
    req.context.insert("current_resource", entry.meta.key);
    req.context.insert("resource", &entry.meta);
    req.context.insert(
        "resources",
        &state.registry.all().map(|e| &e.meta).collect::<Vec<_>>(),
    );

    for (k, v) in &entry.meta.extra_context {
        req.context.insert(k, v);
    }
}

/// Vérifie le token CSRF depuis le body du formulaire.
/// Le middleware délègue la validation de form à Prisme — on la fait manuellement ici.
fn check_csrf(body: &StrMap, session_token: &str) -> AppResult<()> {
    let submitted = body.get("csrf_token").map(|s| s.as_str());
    if submitted != Some(session_token) {
        return Err(Box::new(AppError::new(ErrorContext::generic(
            StatusCode::FORBIDDEN,
            t("csrf.invalid_or_missing").as_ref(),
        ))));
    }
    Ok(())
}

/// Convertit un `Value::Object` en `StrMap` pour pré-remplir un formulaire.
fn value_to_strmap(v: Value) -> StrMap {
    let mut map = StrMap::new();
    if let Value::Object(obj) = v {
        for (k, v) in obj {
            let s = match v {
                Value::Null => String::new(),
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                other => other.to_string(),
            };
            map.insert(k, s);
        }
    }
    map
}

async fn handle_list(
    req: &mut Request,
    entry: &crate::admin::ResourceEntry,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let entries = match &entry.list_fn {
        Some(f) => f(req.engine.db.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?,
        None => Vec::new(),
    };
    req.context.insert("lang", &current_lang().code());
    req.context.insert("entries", &entries);
    req.context.insert("total", &entries.len());
    req.context.insert("current_page", "list");
    let template = entry
        .meta
        .template_list
        .as_deref()
        .unwrap_or_else(|| state.config.templates.list.resolve());
    req.render(template)
}

async fn handle_create_get(
    req: &mut Request,
    entry: &crate::admin::ResourceEntry,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let tera = req.engine.tera.clone();
    let csrf = req.csrf_token.as_str().to_string();
    let form = (entry.form_builder)(StrMap::new(), tera, csrf, axum::http::Method::GET).await;

    req.context.insert("form_fields", form.get_form());
    req.context.insert("is_edit", &false);
    req.context.insert("lang", &current_lang().code());
    let template = entry
        .meta
        .template_create
        .as_deref()
        .unwrap_or_else(|| state.config.templates.create.resolve());
    req.render(template)
}

async fn handle_create_post(
    req: &mut Request,
    entry: &crate::admin::ResourceEntry,
    body: StrMap,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    check_csrf(&body, req.csrf_token.as_str())?;
    let body_for_create = body.clone();
    let tera = req.engine.tera.clone();
    let csrf = req.csrf_token.as_str().to_string();
    let mut form = (entry.form_builder)(body, tera, csrf, axum::http::Method::POST).await;

    if form.is_valid().await {
        let result = match &entry.create_fn {
            Some(f) => f(req.engine.db.clone(), body_for_create).await,
            None => form.save(&req.engine.db).await,
        };
        if let Err(e) = result {
            form.get_form_mut().database_error(&e);
            return Err(Box::new(AppError::new(ErrorContext::database(e))));
        }
        flash_now!(success => t("admin.create.success").to_string());
        return Ok(Redirect::to(&format!(
            "{}/{}/list",
            state.config.prefix.trim_end_matches('/'),
            entry.meta.key
        ))
        .into_response());
    }

    req.context.insert("form_fields", form.get_form());
    req.context.insert("is_edit", &false);
    req.context.insert("lang", &current_lang().code());
    let template = entry
        .meta
        .template_create
        .as_deref()
        .unwrap_or_else(|| state.config.templates.create.resolve());
    req.render(template)
}

async fn handle_detail(
    req: &mut Request,
    entry: &crate::admin::ResourceEntry,
    id: String,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let object = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?,
        None => None,
    };

    if let Some(v) = &object {
        req.context.insert("entry", v);
    }
    req.context.insert("object_id", &id);
    let template = entry
        .meta
        .template_detail
        .as_deref()
        .unwrap_or_else(|| state.config.templates.detail.resolve());
    req.render(template)
}

async fn handle_edit_get(
    req: &mut Request,
    entry: &crate::admin::ResourceEntry,
    id: String,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let tera = req.engine.tera.clone();
    let csrf = req.csrf_token.as_str().to_string();

    // Pré-remplissage via get_fn si disponible
    let data = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?
            .map(value_to_strmap)
            .unwrap_or_default(),
        None => StrMap::new(),
    };

    let builder = entry
        .edit_form_builder
        .as_ref()
        .unwrap_or(&entry.form_builder);
    let form = (builder)(data, tera, csrf, axum::http::Method::GET).await;

    req.context.insert("form_fields", form.get_form());
    req.context.insert("is_edit", &true);
    req.context.insert("object_id", &id);
    let template = entry
        .meta
        .template_edit
        .as_deref()
        .unwrap_or_else(|| state.config.templates.edit.resolve());
    req.render(template)
}

async fn handle_edit_post(
    req: &mut Request,
    entry: &crate::admin::ResourceEntry,
    id: String,
    body: StrMap,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    check_csrf(&body, req.csrf_token.as_str())?;
    let body_for_update = body.clone();
    let tera = req.engine.tera.clone();
    let csrf = req.csrf_token.as_str().to_string();
    let builder = entry
        .edit_form_builder
        .as_ref()
        .unwrap_or(&entry.form_builder);
    // Method::PATCH signals edit mode — password fields relax their required constraint
    // (empty password = keep existing, handled by NotSet in admin_from_form)
    let mut form = (builder)(body, tera, csrf, axum::http::Method::PATCH).await;

    if form.is_valid().await {
        let result = match &entry.update_fn {
            Some(f) => f(req.engine.db.clone(), id, body_for_update).await,
            None => form.save(&req.engine.db).await,
        };
        if let Err(e) = result {
            form.get_form_mut().database_error(&e);
            return Err(Box::new(AppError::new(ErrorContext::database(e))));
        }
        flash_now!(success => t("admin.edit.success").to_string());
        return Ok(Redirect::to(&format!(
            "{}/{}/list",
            state.config.prefix.trim_end_matches('/'),
            entry.meta.key
        ))
        .into_response());
    }

    req.context.insert("form_fields", form.get_form());
    req.context.insert("is_edit", &true);
    req.context.insert("object_id", &id);
    let template = entry
        .meta
        .template_edit
        .as_deref()
        .unwrap_or_else(|| state.config.templates.edit.resolve());
    req.render(template)
}

async fn handle_delete_get(
    req: &mut Request,
    entry: &crate::admin::ResourceEntry,
    id: String,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let object = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?,
        None => None,
    };

    if let Some(v) = &object {
        req.context.insert("entry", v);
    }
    req.context.insert("object_id", &id);
    let template = entry
        .meta
        .template_delete
        .as_deref()
        .unwrap_or_else(|| state.config.templates.delete.resolve());
    req.render(template)
}

async fn handle_delete_post(
    req: &mut Request,
    entry: &crate::admin::ResourceEntry,
    id: String,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let delete_fn = entry.delete_fn.as_ref().ok_or_else(|| {
        Box::new(AppError::new(ErrorContext::not_found(
            "delete_fn non configurée pour cette ressource",
        )))
    })?;

    delete_fn(req.engine.db.clone(), id)
        .await
        .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;

    flash_now!(success => t("admin.delete.success").to_string());
    Ok(Redirect::to(&format!(
        "{}/{}/list",
        state.config.prefix.trim_end_matches('/'),
        entry.meta.key
    ))
    .into_response())
}
