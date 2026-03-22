// ═══════════════════════════════════════════════════════════════
// admin_main — Handler générique central de l'interface admin
//
// Routes :
//   /admin/{resource}/{action}          → admin_get / admin_post
//   /admin/{resource}/{id}/{action}     → admin_get_id / admin_post_id
// ═══════════════════════════════════════════════════════════════

use std::sync::Arc;

use axum::{
    Extension,
    body::Body,
    extract::{FromRequest, Path, Query},
    http::{Request as HttpRequest, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use serde_json::Value;

use crate::admin::AdminRegistry;
use crate::admin::config::AdminConfig;
use crate::admin::resource::ColumnFilter;
use crate::admin::resource_entry::{ListParams, SortDir};
use crate::admin::trad::insert_admin_messages;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::flash_now;
use crate::forms::prisme::aegis;
use crate::utils::aliases::{ARuniqueConfig, AppResult, StrMap};
use crate::utils::trad::{current_lang, t};
use subtle::ConstantTimeEq;

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
    Query(params): Query<StrMap>,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry);

    match action.as_str() {
        "list" => {
            let page = params
                .get("page")
                .and_then(|p| p.parse::<u64>().ok())
                .unwrap_or(1)
                .max(1);
            let sort_by = params.get("sort_by").filter(|s| !s.is_empty()).cloned();
            let sort_dir = match params.get("sort_dir").map(|s| s.as_str()) {
                Some("desc") => SortDir::Desc,
                _ => SortDir::Asc,
            };
            let search = params.get("search").filter(|s| !s.is_empty()).cloned();
            let column_filters: Vec<(String, String)> = params
                .iter()
                .filter_map(|(k, v)| {
                    k.strip_prefix("filter_")
                        .filter(|_| !v.is_empty())
                        .map(|col| (col.to_string(), v.clone()))
                })
                .collect();
            handle_list(
                &mut req,
                entry,
                &state,
                page,
                sort_by,
                sort_dir,
                search,
                column_filters,
            )
            .await
        }
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
    req.context.insert("site_url", &state.config.site_url);
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

    let registered_roles = crate::admin::get_roles();
    req.context.insert("registered_roles", &registered_roles);
}

/// Vérifie le token CSRF depuis le body du formulaire.
/// Le middleware délègue la validation de form à Prisme — on la fait manuellement ici.
fn check_csrf(body: &StrMap, session_token: &str) -> AppResult<()> {
    let valid = body
        .get("csrf_token")
        .map(|s| bool::from(s.as_bytes().ct_eq(session_token.as_bytes())))
        .unwrap_or(false);
    if !valid {
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
    page: u64,
    sort_by: Option<String>,
    sort_dir: SortDir,
    search: Option<String>,
    column_filters: Vec<(String, String)>,
) -> AppResult<Response> {
    let page_size = state.config.page_size;
    let offset = (page - 1) * page_size;

    let list_params = ListParams {
        offset,
        limit: page_size,
        sort_by: sort_by.clone(),
        sort_dir: sort_dir.clone(),
        search: search.clone(),
        column_filters: column_filters.clone(),
    };

    let (entries_result, count_result, filter_values_result) = tokio::join!(
        async {
            match &entry.list_fn {
                Some(f) => f(req.engine.db.clone(), list_params).await,
                None => Ok(Vec::new()),
            }
        },
        async {
            match &entry.count_fn {
                Some(f) => f(req.engine.db.clone(), search.clone()).await,
                None => Ok(0u64),
            }
        },
        async {
            match &entry.filter_fn {
                Some(f) => f(req.engine.db.clone()).await.unwrap_or_default(),
                None => std::collections::HashMap::new(),
            }
        }
    );
    let entries = entries_result.map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
    let count = count_result.map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
    let filter_values = filter_values_result;
    // Si count_fn absent, estime le total depuis la page courante (évite la pagination cassée)
    let total = if entry.count_fn.is_some() {
        count
    } else {
        offset + entries.len() as u64
    };

    let page_count = (total + page_size - 1) / page_size;
    let page = page.min(page_count.max(1));

    // Colonnes visibles : toutes sauf id/password, filtrées par DisplayConfig
    let all_cols: Vec<String> = entries
        .first()
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.keys()
                .filter(|k| *k != "id" && !k.starts_with("password"))
                .cloned()
                .collect()
        })
        .unwrap_or_default();

    let (visible_columns, column_labels): (Vec<String>, std::collections::HashMap<String, String>) =
        match &entry.meta.display.columns {
            ColumnFilter::All => (all_cols, std::collections::HashMap::new()),
            ColumnFilter::Include(cols) => {
                let filtered: Vec<(String, String)> = cols
                    .iter()
                    .filter(|(c, _)| all_cols.contains(c))
                    .cloned()
                    .collect();
                let labels = filtered
                    .iter()
                    .map(|(c, l)| (c.clone(), l.clone()))
                    .collect();
                (filtered.into_iter().map(|(c, _)| c).collect(), labels)
            }
            ColumnFilter::Exclude(excluded) => (
                all_cols
                    .into_iter()
                    .filter(|c| !excluded.contains(c))
                    .collect(),
                std::collections::HashMap::new(),
            ),
        };

    // Validation whitelist : sort_by doit être une colonne visible ou "id"
    let safe_sort_by = sort_by
        .filter(|s| s == "id" || visible_columns.contains(s))
        .unwrap_or_default();

    // active_filters : toutes les colonnes list_filter initialisées à "" puis écrasées si actives
    // (évite l'erreur Tera "key not found" lors de l'accès active_filters[col])
    let mut active_filters: std::collections::HashMap<String, String> = entry
        .meta
        .display
        .list_filter
        .iter()
        .map(|(col, _)| (col.clone(), String::new()))
        .collect();
    for (col, val) in &column_filters {
        active_filters.insert(col.clone(), val.clone());
    }

    // filter_qs : query string à ajouter aux liens de pagination / recherche
    let filter_qs: String = column_filters
        .iter()
        .map(|(col, val)| format!("&filter_{}={}", col, urlencoding::encode(val)))
        .collect();

    req.context.insert("lang", &current_lang().code());
    req.context.insert("entries", &entries);
    req.context.insert("total", &total);
    req.context.insert("page", &page);
    req.context.insert("page_count", &page_count);
    req.context.insert("has_prev", &(page > 1));
    req.context.insert("has_next", &(page < page_count));
    req.context.insert("prev_page", &(page.saturating_sub(1)));
    req.context.insert("next_page", &(page + 1));
    req.context.insert("current_page", "list");
    req.context.insert("visible_columns", &visible_columns);
    req.context.insert("column_labels", &column_labels);
    req.context.insert("sort_by", &safe_sort_by);
    req.context.insert("sort_dir", &sort_dir.as_str());
    req.context.insert("sort_dir_toggle", &sort_dir.toggle());
    req.context.insert("search", &search.unwrap_or_default());
    req.context.insert("filter_values", &filter_values);
    req.context.insert("active_filters", &active_filters);
    req.context.insert("filter_qs", &filter_qs);

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
