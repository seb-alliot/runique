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
    parse_date_time::format_datetime,
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
    if entry.meta.inject_password && body.get("password").is_some_and(|p| p.is_empty()) {
        let temp_pw = uuid::Uuid::new_v4().to_string();
        if let Ok(hash) = crate::utils::password::hash(&temp_pw) {
            body.insert("password".to_string(), hash);
        }
    }

    let body_for_create = body.clone();
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
    let mut form = (entry.form_builder)(
        req.engine.db.clone(),
        resource_keys,
        body,
        tera,
        csrf,
        axum::http::Method::POST,
    )
    .await;

    if form.is_valid().await {
        let result = match &entry.create_fn {
            Some(f) => f(req.engine.db.clone(), body_for_create.clone()).await,
            None => form.save(&req.engine.db).await,
        };
        match result {
            Ok(()) => {}
            Err(sea_orm::DbErr::Custom(ref msg)) => {
                form.get_form_mut().errors.push(msg.clone());
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
            current_user.id,
            &current_user.username,
            entry.meta.key,
            "",
            "create",
            None,
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
        let summary = old_obj
            .as_ref()
            .and_then(|v| history::diff_fields(v, &body_for_update));
        let result = match &entry.update_fn {
            Some(f) => f(req.engine.db.clone(), id.clone(), body_for_update).await,
            None => form.save(&req.engine.db).await,
        };
        if let Err(e) = result {
            form.get_form_mut().database_error(&e);
            if !is_unique_violation(&e) {
                return Err(Box::new(AppError::new(ErrorContext::database(e))));
            }
        } else {
            history::log_admin_action(
                &req.engine.db,
                current_user.id,
                &current_user.username,
                entry.meta.key,
                &id,
                "edit",
                summary,
            )
            .await;
            req.notices
                .success(t("admin.edit.success").to_string())
                .await;
            return Ok(Redirect::to(&format!(
                "{}/{}/list",
                state.config.prefix.trim_end_matches('/'),
                entry.meta.key
            ))
            .into_response());
        }
    }

    if let Some(ts) = orig_updated_at {
        req.context.insert("orig_updated_at", &ts);
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
        current_user.id,
        &current_user.username,
        entry.meta.key,
        &id,
        "delete",
        None,
    )
    .await;

    req.notices
        .success(t("admin.delete.success").to_string())
        .await;
    Ok(Redirect::to(&format!(
        "{}/{}/list",
        state.config.prefix.trim_end_matches('/'),
        entry.meta.key
    ))
    .into_response())
}
