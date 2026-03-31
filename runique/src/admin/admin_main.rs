// ═══════════════════════════════════════════════════════════════
// admin_main — Handler générique central de l'interface admin
//
// Routes :
//   /admin/{resource}/{action}          → admin_get / admin_post
//   /admin/{resource}/{id}/{action}     → admin_get_id / admin_post_id
// ═══════════════════════════════════════════════════════════════
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::flash_now;
use crate::forms::prisme::aegis;
use crate::utils::{
    CSRF_TOKEN_KEY,
    aliases::{ARuniqueConfig, AppResult, StrMap},
    constante::admin_ctx::{
        common as ctx_common, create as ctx_create, detail as ctx_detail, edit as ctx_edit,
        list as list_ctx,
    },
    trad::{current_lang, t},
};
use crate::{
    admin::{
        AdminRegistry,
        config::AdminConfig,
        resource::ColumnFilter,
        resource_entry::{ListParams, SortDir},
        trad::insert_admin_messages,
    },
    utils::admin_ctx::list::{PAGE, SORT_BY, SORT_DIR},
};
use axum::{
    Extension,
    body::Body,
    extract::{FromRequest, Path, Query},
    http::{Request as HttpRequest, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use subtle::ConstantTimeEq;

// ─── ListQuery — paramètres de la vue liste admin ─────────────

struct ListQuery {
    page: u64,
    sort_by: Option<String>,
    sort_dir: SortDir,
    search: Option<String>,
    column_filters: Vec<(String, String)>,
    filter_pages: HashMap<String, u64>,
}

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
    headers: axum::http::HeaderMap,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry);

    match action.as_str() {
        "list" => {
            let page = params
                .get(PAGE)
                .and_then(|p| p.parse::<u64>().ok())
                .unwrap_or(1)
                .max(1);
            let sort_by = params.get(SORT_BY).filter(|s| !s.is_empty()).cloned();
            let sort_dir = match params.get(SORT_DIR).map(String::as_str) {
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
            let filter_pages: HashMap<String, u64> = params
                .iter()
                .filter_map(|(k, v)| {
                    let col = k.strip_prefix("fp_")?;
                    let page = v.parse::<u64>().ok()?;
                    Some((col.to_string(), page))
                })
                .collect();
            let query = ListQuery {
                page,
                sort_by,
                sort_dir,
                search,
                column_filters,
                filter_pages,
            };
            let is_htmx = headers.contains_key("hx-request");
            handle_list(&mut req, entry, &state, query, is_htmx).await
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
    req.context.insert(ctx_common::LANG, &current_lang().code());
    check_csrf(&body, req.csrf_token.as_str())?;

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
    req.context.insert(ctx_common::LANG, &current_lang().code());
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
    headers: axum::http::HeaderMap,
    Path((resource_key, id, action)): Path<(String, String, String)>,
    Extension(state): Extension<Arc<PrototypeAdminState>>,
    AdminBody(body): AdminBody,
) -> AppResult<Response> {
    let entry = state
        .registry
        .get(&resource_key)
        .ok_or_else(|| Box::new(AppError::new(ErrorContext::not_found("Resource not found"))))?;

    inject_context(&mut req, &state, entry);
    req.context.insert(ctx_common::LANG, &current_lang().code());
    check_csrf(&body, req.csrf_token.as_str())?;

    match action.as_str() {
        "edit" => handle_edit_post(&mut req, entry, id, body, &state).await,
        "delete" => handle_delete_post(&mut req, entry, id, &state).await,
        "reset-password" => handle_reset_password(&mut req, entry, id, &headers, &state).await,
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
    for item in ["list", "create", "edit", "detail", "delete", "base"] {
        insert_admin_messages(&mut req.context, item);
    }

    req.context
        .insert(ctx_common::SITE_TITLE, &state.config.site_title);
    req.context
        .insert(ctx_common::SITE_URL, &state.config.site_url);
    req.context.insert(ctx_common::RESOURCE_KEY, entry.meta.key);
    req.context
        .insert(ctx_common::CURRENT_RESOURCE, entry.meta.key);
    req.context.insert(ctx_common::RESOURCE, &entry.meta);
    req.context.insert(
        ctx_common::RESOURCES,
        &state.registry.all().map(|e| &e.meta).collect::<Vec<_>>(),
    );

    for (k, v) in &entry.meta.extra_context {
        req.context.insert(k, v);
    }

    let registered_roles = crate::admin::get_roles();
    req.context
        .insert(ctx_common::REGISTERED_ROLES, &registered_roles);
}

/// Vérifie le token CSRF depuis le body du formulaire.
/// Le middleware délègue la validation de form à Prisme — on la fait manuellement ici.
fn check_csrf(body: &StrMap, session_token: &str) -> AppResult<()> {
    let valid = body
        .get(CSRF_TOKEN_KEY)
        .map(|s| {
            if let Ok(unmasked) = crate::utils::csrf::unmask_csrf_token(s) {
                bool::from(unmasked.as_bytes().ct_eq(session_token.as_bytes()))
            } else {
                bool::from(s.as_bytes().ct_eq(session_token.as_bytes()))
            }
        })
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
                // Arrays et objets imbriqués ne peuvent pas pré-remplir un champ plat
                Value::Array(_) | Value::Object(_) => continue,
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
    query: ListQuery,
    is_htmx: bool,
) -> AppResult<Response> {
    let ListQuery {
        page,
        sort_by,
        sort_dir,
        search,
        column_filters,
        filter_pages,
    } = query;
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

    let (entries_result, count_result, filter_result) = tokio::join!(
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
                Some(f) => f(req.engine.db.clone(), filter_pages.clone())
                    .await
                    .unwrap_or_else(|e| {
                        if let Some(level) = crate::utils::runique_log::get_log().filter_fn {
                            crate::runique_log!(level, resource = entry.meta.key, error = %e, "filter_fn a échoué — liste retournée sans filtres sidebar");
                        }
                        HashMap::new()
                    }),
                None => HashMap::new(),
            }
        }
    );
    let entries = entries_result.map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;
    let count = count_result.map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;

    // Séparer valeurs et totaux distincts par colonne
    let filter_values: HashMap<String, Vec<String>> = filter_result
        .iter()
        .map(|(k, (vals, _))| (k.clone(), vals.clone()))
        .collect();
    let filter_totals: HashMap<String, u64> = filter_result
        .into_iter()
        .map(|(k, (_, total))| (k, total))
        .collect();
    // Si count_fn absent, estime le total depuis la page courante (évite la pagination cassée)
    let total = if entry.count_fn.is_some() {
        count
    } else {
        offset + entries.len() as u64
    };

    let page_count = total.div_ceil(page_size);
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

    let (visible_columns, column_labels): (Vec<String>, HashMap<String, String>) =
        match &entry.meta.display.columns {
            ColumnFilter::All => (all_cols, HashMap::new()),
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
                HashMap::new(),
            ),
        };

    // Validation whitelist : sort_by doit être une colonne visible ou "id"
    let safe_sort_by = sort_by
        .filter(|s| s == "id" || visible_columns.contains(s))
        .unwrap_or_default();

    // active_filters : toutes les colonnes list_filter initialisées à "" puis écrasées si actives
    // (évite l'erreur Tera "key not found" lors de l'accès active_filters[col])
    let mut active_filters: HashMap<String, String> = entry
        .meta
        .display
        .list_filter
        .iter()
        .map(|(col, _, _)| (col.clone(), String::new()))
        .collect();
    for (col, val) in &column_filters {
        active_filters.insert(col.clone(), val.clone());
    }

    // filter_qs : query string ajouté aux liens de pagination / tri / recherche
    // Inclut les filtres actifs ET les pages de sidebar (fp_*) pour les préserver
    let filter_qs: String = {
        let mut parts: Vec<String> = column_filters
            .iter()
            .map(|(col, val)| format!("&filter_{}={}", col, urlencoding::encode(val)))
            .collect();
        for (col, page) in &filter_pages {
            if *page > 0 {
                parts.push(format!("&fp_{}={}", col, page));
            }
        }
        parts.concat()
    };

    // filter_meta : prev/next QS précalculés par colonne pour la pagination sidebar
    let base_qs: Vec<String> = {
        let mut parts = vec![];
        if !safe_sort_by.is_empty() {
            parts.push(format!("sort_by={}", safe_sort_by));
        }
        if sort_dir == SortDir::Desc {
            parts.push("sort_dir=desc".to_string());
        }
        if let Some(ref s) = search {
            parts.push(format!("search={}", urlencoding::encode(s)));
        }
        for (col, val) in &column_filters {
            parts.push(format!("filter_{}={}", col, urlencoding::encode(val)));
        }
        parts
    };
    let filter_meta: HashMap<String, serde_json::Value> = entry
        .meta
        .display
        .list_filter
        .iter()
        .map(|(col, _, col_limit)| {
            let cur_page = filter_pages.get(col).copied().unwrap_or(0);
            let total_distinct = filter_totals.get(col).copied().unwrap_or(0);
            let total_pages = total_distinct.div_ceil(*col_limit);
            let total_pages = total_pages.max(1);
            let has_prev = cur_page > 0;
            let has_next = cur_page + 1 < total_pages;

            let build_qs = |fp_override: Option<u64>| -> String {
                let mut parts = base_qs.clone();
                for (other_col, other_page) in &filter_pages {
                    if other_col != col && *other_page > 0 {
                        parts.push(format!("fp_{}={}", other_col, other_page));
                    }
                }
                if let Some(fp) = fp_override {
                    if fp > 0 {
                        parts.push(format!("fp_{}={}", col, fp));
                    }
                }
                parts.join("&")
            };

            let prev_qs = if has_prev {
                build_qs(Some(cur_page - 1))
            } else {
                String::new()
            };
            let next_qs = if has_next {
                build_qs(Some(cur_page + 1))
            } else {
                String::new()
            };

            let meta = serde_json::json!({
                "current_page": cur_page,
                "total_pages": total_pages,
                "has_prev": has_prev,
                "has_next": has_next,
                "prev_qs": prev_qs,
                "next_qs": next_qs,
            });
            (col.clone(), meta)
        })
        .collect();

    macro_rules! ctx {
        ($($key:expr => $val:expr),* $(,)?) => {
            $( req.context.insert($key, &$val); )*
        };
    }

    ctx! {
        list_ctx::LANG              => current_lang().code(),
        list_ctx::ENTRIES           => entries,
        list_ctx::TOTAL             => total,
        list_ctx::PAGE              => page,
        list_ctx::PAGE_COUNT        => page_count,
        list_ctx::HAS_PREV          => (page > 1),
        list_ctx::HAS_NEXT          => (page < page_count),
        list_ctx::PREV_PAGE         => page.saturating_sub(1),
        list_ctx::NEXT_PAGE         => (page + 1),
        "current_page"              => "list",
        list_ctx::VISIBLE_COLUMNS   => visible_columns,
        list_ctx::COLUMN_LABELS     => column_labels,
        list_ctx::SORT_BY           => safe_sort_by,
        list_ctx::SORT_DIR          => sort_dir.as_str(),
        list_ctx::SORT_DIR_TOGGLE   => sort_dir.toggle(),
        list_ctx::SEARCH            => search.unwrap_or_default(),
        list_ctx::FILTER_VALUES     => filter_values,
        list_ctx::ACTIVE_FILTERS    => active_filters,
        list_ctx::FILTER_QS         => filter_qs,
        list_ctx::FILTER_META       => filter_meta,
    }

    let template = if is_htmx {
        "admin/list_partial"
    } else {
        entry
            .meta
            .template_list
            .as_deref()
            .unwrap_or_else(|| state.config.templates.list.resolve())
    };
    req.render(template)
}

async fn handle_create_get(
    req: &mut Request,
    entry: &crate::admin::ResourceEntry,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    let tera = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();
    let form = (entry.form_builder)(StrMap::new(), tera, csrf, axum::http::Method::GET).await;

    req.context.insert(ctx_create::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_create::IS_EDIT, &false);
    req.context.insert(ctx_common::LANG, &current_lang().code());
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
    let body_for_create = body.clone();
    let tera = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();
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

    req.context.insert(ctx_create::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_create::IS_EDIT, &false);
    req.context.insert(ctx_common::LANG, &current_lang().code());
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
        req.context.insert(ctx_detail::ENTRY, v);
    }
    req.context.insert(ctx_detail::OBJECT_ID, &id);
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
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();

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

    req.context.insert(ctx_edit::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_edit::IS_EDIT, &true);
    req.context.insert(ctx_edit::OBJECT_ID, &id);
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
    let body_for_update = body.clone();
    let tera = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();
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

    req.context.insert(ctx_edit::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_edit::IS_EDIT, &true);
    req.context.insert(ctx_edit::OBJECT_ID, &id);
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
        req.context.insert(ctx_detail::ENTRY, v);
    }
    req.context.insert(ctx_detail::OBJECT_ID, &id);
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
        println!{"sa bug ici, admin_main ligne 784"}
        Box::new(AppError::new(ErrorContext::not_found(
            t("admin.delete.not_found").as_ref(),
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

// ─── Reset password ──────────────────────────────────────────────────────────

async fn handle_reset_password(
    req: &mut Request,
    entry: &crate::admin::ResourceEntry,
    id: String,
    headers: &axum::http::HeaderMap,
    state: &PrototypeAdminState,
) -> AppResult<Response> {
    // Récupère l'entrée pour extraire l'email
    let object = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?,
        None => None,
    };

    let detail_url = format!(
        "{}/{}/{}/detail",
        state.config.prefix.trim_end_matches('/'),
        entry.meta.key,
        id
    );

    let fields = object.as_ref().and_then(|v| v.as_object());

    let email = fields
        .and_then(|m| m.get("email"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let Some(email) = email else {
        req.notices
            .error(t("admin.reset_password.error_no_email"))
            .await;
        return Ok(Redirect::to(&detail_url).into_response());
    };

    let username = fields
        .and_then(|m| m.get("username").or_else(|| m.get("name")))
        .and_then(|v| v.as_str())
        .unwrap_or(&email);

    let token = crate::utils::reset_token::generate(&email);
    let encrypted_email = crate::utils::reset_token::encrypt_email(&token, &email);

    let reset_url = if let Some(base) = &state.config.reset_password_url {
        format!(
            "{}/{}/{}",
            base.trim_end_matches('/'),
            token,
            encrypted_email
        )
    } else {
        let host = headers
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");
        format!(
            "http://{}/reset-password/{}/{}",
            host, token, encrypted_email
        )
    };

    // Envoi par email si mailer configuré, sinon affiche le lien dans le flash
    if crate::utils::mailer_configured() {
        let body = format!(
            "<p>Bonjour {username},</p><p>Cliquez sur le lien suivant pour réinitialiser votre mot de passe (valide 1 heure) :</p><p><a href=\"{reset_url}\">{reset_url}</a></p>"
        );
        match crate::utils::Email::new()
            .to(email.clone())
            .subject(t("admin.reset_password.btn"))
            .html(body)
            .send()
            .await
        {
            Ok(_) => {
                req.notices
                    .success(crate::utils::trad::tf(
                        "admin.reset_password.success_email",
                        &[&email],
                    ))
                    .await;
            }
            Err(e) => {
                req.notices
                    .error(crate::utils::trad::tf(
                        "admin.reset_password.error_send",
                        &[&e],
                    ))
                    .await;
            }
        }
    } else {
        req.notices
            .success(crate::utils::trad::tf(
                "admin.reset_password.success_link",
                &[&reset_url],
            ))
            .await;
    }

    Ok(Redirect::to(&detail_url).into_response())
}
