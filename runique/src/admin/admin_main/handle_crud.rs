use super::format_datetime;
use crate::admin::helper::resource_entry::ResourceEntry;
use crate::admin::history;
use crate::auth::session::CurrentUser;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::{
    aliases::{AppResult, StrMap},
    constante::admin_context::{
        common as ctx_common, create as ctx_create, detail as ctx_detail, edit as ctx_edit,
    },
    trad::{current_lang, t},
};
use axum::response::{IntoResponse, Redirect, Response};
use serde_json::Value;

fn is_unique_violation(e: &sea_orm::DbErr) -> bool {
    let msg = e.to_string();
    msg.contains("unique") || msg.contains("UNIQUE") || msg.contains("Duplicate")
}

pub(super) fn value_to_strmap(v: Value) -> StrMap {
    let mut map = StrMap::new();
    if let Value::Object(obj) = v {
        for (k, v) in obj {
            let s = match v {
                Value::Null => String::new(),
                Value::String(s) => s,
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Array(_) | Value::Object(_) => continue,
            };
            map.insert(k, s);
        }
    }
    map
}

pub(super) async fn handle_detail(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    state: &super::PrototypeAdminState,
) -> AppResult<Response> {
    let object = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?,
        None => None,
    };

    if let Some(mut v) = object {
        format_datetime(&mut v);
        req.context.insert(ctx_detail::ENTRY, &v);
    }
    req.context.insert(ctx_detail::OBJECT_ID, &id);
    let template = entry
        .meta
        .template_detail
        .as_deref()
        .unwrap_or_else(|| state.config.templates.detail.resolve());
    req.render(template)
}

pub(super) async fn handle_create_get(
    req: &mut Request,
    entry: &ResourceEntry,
    state: &super::PrototypeAdminState,
) -> AppResult<Response> {
    let tera = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();
    let resource_keys = state
        .registry
        .all()
        .map(|e| e.meta.key.to_string())
        .collect::<Vec<_>>();
    let form = (entry.form_builder)(
        req.engine.db.clone(),
        resource_keys,
        StrMap::new(),
        tera,
        csrf,
        axum::http::Method::GET,
    )
    .await;

    if let Some(loader) = &entry.m2m_loader {
        let m2m_fields = loader(req.engine.db.clone(), None).await;
        req.context.insert("m2m_fields", &m2m_fields);
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

pub(super) async fn handle_create_post(
    req: &mut Request,
    entry: &ResourceEntry,
    mut body: StrMap,
    headers: &axum::http::HeaderMap,
    state: &super::PrototypeAdminState,
    current_user: &CurrentUser,
) -> AppResult<Response> {
    eprintln!("[DEBUG create_post] resource={}", entry.meta.key);
    if entry.meta.inject_password && body.get("password").is_some_and(|p| p.is_empty()) {
        let temp_pw = uuid::Uuid::new_v4().to_string();
        if let Ok(hash) = crate::utils::password::hash(&temp_pw) {
            body.insert("password".to_string(), hash);
        }
    }

    let mut body_for_create = body.clone();
    let tera = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();
    let resource_keys = state
        .registry
        .all()
        .map(|e| e.meta.key.to_string())
        .collect::<Vec<_>>();
    eprintln!("[DEBUG create_post] building form...");
    let mut form = (entry.form_builder)(
        req.engine.db.clone(),
        resource_keys,
        body,
        tera,
        csrf,
        axum::http::Method::POST,
    )
    .await;
    eprintln!("[DEBUG create_post] form built");

    let valid = form.is_valid().await;
    eprintln!(
        "[DEBUG create_post] is_valid={valid}, errors={:?}",
        form.get_form().errors
    );
    if valid {
        // Sync finalized field values (e.g. file paths moved by finalize()) into body
        for (name, field) in &form.get_form().fields {
            body_for_create.insert(name.clone(), field.value().to_string());
        }
        eprintln!("[DEBUG create_post] calling create_fn...");
        let result = match &entry.create_fn {
            Some(f) => f(req.engine.db.clone(), body_for_create.clone()).await,
            None => form.save(&req.engine.db).await,
        };
        eprintln!(
            "[DEBUG create_post] create_fn result: {:?}",
            result.as_ref().map(|_| "ok").map_err(|e| e.to_string())
        );
        match result {
            Ok(()) => {}
            Err(sea_orm::DbErr::Custom(ref msg)) => {
                form.get_form_mut().errors.push(msg.clone());
                if let Some(loader) = &entry.m2m_loader {
                    let m2m_fields = loader(req.engine.db.clone(), None).await;
                    req.context.insert("m2m_fields", &m2m_fields);
                }
                req.context.insert(ctx_create::FORM_FIELDS, form.get_form());
                req.context.insert(ctx_create::IS_EDIT, &false);
                req.context.insert(ctx_common::LANG, &current_lang().code());
                let template = entry
                    .meta
                    .template_create
                    .as_deref()
                    .unwrap_or_else(|| state.config.templates.create.resolve());
                return req.render(template);
            }
            Err(e) => {
                form.get_form_mut().database_error(&e);
                if !is_unique_violation(&e) {
                    return Err(Box::new(AppError::new(ErrorContext::database(e))));
                }
                if let Some(loader) = &entry.m2m_loader {
                    let m2m_fields = loader(req.engine.db.clone(), None).await;
                    req.context.insert("m2m_fields", &m2m_fields);
                }
                req.context.insert(ctx_create::FORM_FIELDS, form.get_form());
                req.context.insert(ctx_create::IS_EDIT, &false);
                req.context.insert(ctx_common::LANG, &current_lang().code());
                let template = entry
                    .meta
                    .template_create
                    .as_deref()
                    .unwrap_or_else(|| state.config.templates.create.resolve());
                return req.render(template);
            }
        }

        history::log_admin_action(
            &req.engine.db,
            history::AdminActionLog {
                user_id: current_user.id,
                username: &current_user.username,
                resource_key: entry.meta.key,
                object_pk: "",
                action: "create",
                summary: None,
                batch_id: None,
            },
        )
        .await;

        if entry.meta.inject_password
            && let Some(email) = body_for_create.get("email")
        {
            let email_template = state
                .config
                .user_resources
                .get(entry.meta.key)
                .and_then(|t| t.as_deref());
            super::handle_password::send_user_created_email(
                req,
                entry,
                email,
                body_for_create.get("username").map(String::as_str),
                email_template,
                headers,
                state,
            )
            .await;
        }

        req.notices
            .success(t("admin.create.success").to_string())
            .await;
        return Ok(Redirect::to(&format!(
            "{}/{}/list",
            state.config.prefix.trim_end_matches('/'),
            entry.meta.key
        ))
        .into_response());
    }

    if let Some(loader) = &entry.m2m_loader {
        let m2m_fields = loader(req.engine.db.clone(), None).await;
        req.context.insert("m2m_fields", &m2m_fields);
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

pub(super) async fn handle_edit_get(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    state: &super::PrototypeAdminState,
) -> AppResult<Response> {
    let tera = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();

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
    let resource_keys = state
        .registry
        .all()
        .map(|e| e.meta.key.to_string())
        .collect::<Vec<_>>();
    let form = (builder)(
        req.engine.db.clone(),
        resource_keys,
        data.clone(),
        tera,
        csrf,
        axum::http::Method::GET,
    )
    .await;

    if let Some(ts) = data.get("updated_at") {
        req.context.insert("orig_updated_at", ts);
    }

    let return_qs = req
        .prisme
        .data
        .get("return_qs")
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_default();

    if let Some(loader) = &entry.m2m_loader {
        let m2m_fields = loader(req.engine.db.clone(), Some(id.clone())).await;
        req.context.insert("m2m_fields", &m2m_fields);
    }

    req.context.insert(ctx_edit::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_edit::IS_EDIT, &true);
    req.context.insert(ctx_edit::OBJECT_ID, &id);
    req.context.insert("return_qs", &return_qs);
    let template = entry
        .meta
        .template_edit
        .as_deref()
        .unwrap_or_else(|| state.config.templates.edit.resolve());
    req.render(template)
}

pub(super) async fn handle_edit_post(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    body: StrMap,
    state: &super::PrototypeAdminState,
    current_user: &CurrentUser,
) -> AppResult<Response> {
    let mut body_for_update = body.clone();
    let orig_updated_at = body_for_update.remove("__original_updated_at");
    let return_qs = body_for_update
        .remove("return_qs")
        .filter(|s| !s.is_empty());

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
    let resource_keys = state
        .registry
        .all()
        .map(|e| e.meta.key.to_string())
        .collect::<Vec<_>>();
    let mut form = (builder)(
        req.engine.db.clone(),
        resource_keys,
        body,
        tera,
        csrf,
        axum::http::Method::PATCH,
    )
    .await;

    let mut is_locked = false;
    let is_form_valid = form.is_valid().await;

    let old_obj: Option<Value> = if is_form_valid {
        if let Some(get_fn) = &entry.get_fn {
            get_fn(req.engine.db.clone(), id.clone())
                .await
                .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?
        } else {
            None
        }
    } else {
        None
    };

    if is_form_valid
        && let Some(orig_ts) = &orig_updated_at
        && let Some(current_obj) = &old_obj
        && let Some(current_ts) = current_obj.get("updated_at").and_then(|v| v.as_str())
        && current_ts != orig_ts
    {
        is_locked = true;
        form.get_form_mut().errors.push("Update failed: This content has been modified by another person during your editing. Please copy your changes and reload the page.".to_string());
        req.notices.error("This content has been modified by someone else during your editing. Refresh the page.").await;
    }

    if !is_locked && !form.get_form().has_errors() {
        // Sync finalized field values (e.g. file paths moved by finalize()) into body
        for (name, field) in &form.get_form().fields {
            body_for_update.insert(name.clone(), field.value().to_string());
        }
        // Delete old files replaced by a new upload
        if let Some(ref old) = old_obj {
            let media_root = std::env::var("MEDIA_ROOT").unwrap_or_else(|_| "media".to_string());
            let media_root = media_root.trim_end_matches('/');
            for (name, field) in &form.get_form().fields {
                if field.field_type() != "file" {
                    continue;
                }
                let new_val = field.value();
                if new_val.is_empty() {
                    continue;
                }
                if let Some(old_val) = old.get(name).and_then(|v| v.as_str())
                    && !old_val.is_empty()
                    && old_val != new_val
                {
                    let old_abs = format!("{}/{}", media_root, old_val.trim_start_matches('/'));
                    let _ = std::fs::remove_file(&old_abs);
                }
            }
        }
        let summary = old_obj
            .as_ref()
            .and_then(|v| history::diff_fields(v, &body_for_update));
        eprintln!("[DEBUG edit_post] calling update_fn for id={id}...");
        let result = match &entry.update_fn {
            Some(f) => f(req.engine.db.clone(), id.clone(), body_for_update).await,
            None => form.save(&req.engine.db).await,
        };
        eprintln!(
            "[DEBUG edit_post] update result: {:?}",
            result.as_ref().map(|_| "ok").map_err(|e| e.to_string())
        );
        if let Err(e) = result {
            form.get_form_mut().database_error(&e);
            if !is_unique_violation(&e) {
                return Err(Box::new(AppError::new(ErrorContext::database(e))));
            }
        } else {
            if summary.is_some() {
                history::log_admin_action(
                    &req.engine.db,
                    history::AdminActionLog {
                        user_id: current_user.id,
                        username: &current_user.username,
                        resource_key: entry.meta.key,
                        object_pk: &id,
                        action: "edit",
                        summary,
                        batch_id: None,
                    },
                )
                .await;
            }
            req.notices
                .success(t("admin.edit.success").to_string())
                .await;
            let list_url = match return_qs {
                Some(qs) => format!(
                    "{}/{}/list?{}",
                    state.config.prefix.trim_end_matches('/'),
                    entry.meta.key,
                    qs
                ),
                None => format!(
                    "{}/{}/list",
                    state.config.prefix.trim_end_matches('/'),
                    entry.meta.key
                ),
            };
            return Ok(Redirect::to(&list_url).into_response());
        }
    }

    if let Some(ts) = orig_updated_at {
        req.context.insert("orig_updated_at", &ts);
    }

    if let Some(loader) = &entry.m2m_loader {
        let m2m_fields = loader(req.engine.db.clone(), Some(id.clone())).await;
        req.context.insert("m2m_fields", &m2m_fields);
    }

    let return_qs_str = return_qs.as_deref().unwrap_or("");
    req.context.insert(ctx_edit::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_edit::IS_EDIT, &true);
    req.context.insert(ctx_edit::OBJECT_ID, &id);
    req.context.insert("return_qs", return_qs_str);
    let template = entry
        .meta
        .template_edit
        .as_deref()
        .unwrap_or_else(|| state.config.templates.edit.resolve());
    req.render(template)
}

pub(super) async fn handle_delete_get(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    state: &super::PrototypeAdminState,
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

pub(super) async fn handle_delete_post(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    state: &super::PrototypeAdminState,
    current_user: &CurrentUser,
) -> AppResult<Response> {
    let delete_fn = entry.delete_fn.as_ref().ok_or_else(|| {
        Box::new(AppError::new(ErrorContext::not_found(
            t("admin.delete.not_found").as_ref(),
        )))
    })?;

    delete_fn(req.engine.db.clone(), id.clone())
        .await
        .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;

    history::log_admin_action(
        &req.engine.db,
        history::AdminActionLog {
            user_id: current_user.id,
            username: &current_user.username,
            resource_key: entry.meta.key,
            object_pk: &id,
            action: "delete",
            summary: None,
            batch_id: None,
        },
    )
    .await;

    req.notices
        .success(t("admin.delete.success").to_string())
        .await;
    let return_qs = req
        .prisme
        .data
        .get("return_qs")
        .filter(|s| !s.is_empty())
        .cloned();
    let list_url = match return_qs {
        Some(qs) => format!(
            "{}/{}/list?{}",
            state.config.prefix.trim_end_matches('/'),
            entry.meta.key,
            qs
        ),
        None => format!(
            "{}/{}/list",
            state.config.prefix.trim_end_matches('/'),
            entry.meta.key
        ),
    };
    Ok(Redirect::to(&list_url).into_response())
}
