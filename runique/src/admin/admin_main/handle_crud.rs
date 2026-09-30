use super::format_datetime;
use super::{ParentBinding, closure_id_of, scope_base};
use crate::admin::helper::resource_entry::ResourceEntry;
use crate::admin::history;
use crate::auth::session::CurrentUser;
use crate::config::static_files::resolve_media_root;
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

fn inject_csp_nonce(
    form: &mut Box<dyn crate::admin::helper::dyn_form::DynForm>,
    ctx: &tera::Context,
) {
    if let Some(nonce) = ctx.get("csp_nonce").and_then(|v| v.as_str()) {
        form.get_form_mut().set_csp_nonce(nonce);
    }
}

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

/// Forces the parent-scope identity columns into submitted data, so a nested
/// create/edit always writes the parent from the (authorized) URL path — never
/// a value the client could tamper with in the hidden field. On edit the local
/// key is also pinned so a composite child's identity can't drift.
fn force_scope_values(data: &mut StrMap, parent: &ParentBinding, local_id: Option<&str>) {
    data.insert(parent.fk_col.to_string(), parent.parent_id.clone());
    if let (Some(col), Some(local)) = (parent.local_key, local_id) {
        data.insert(col.to_string(), local.to_string());
    }
}

/// What a create/update function receives: the values the form validated
/// (after `finalize`), plus the many-to-many checkboxes (`m2m_<field>__<id>`),
/// which the templates render outside the form and the generated code checks.
/// Any other key the client added to the request is dropped: a column the form
/// doesn't show must not become writable by adding it to the request body.
fn accepted_data(form: &crate::forms::Forms, body: &StrMap) -> StrMap {
    let mut data: StrMap = form
        .fields
        .iter()
        .map(|(name, field)| (name.clone(), field.value().to_string()))
        .collect();
    data.extend(
        body.iter()
            .filter(|(key, _)| key.starts_with("m2m_"))
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    data
}

/// For a resource that creates accounts through the reset-email flow
/// (`inject_password`): the password is a random one nobody knows, unless the
/// form has a real password input the admin typed into. A hidden or missing
/// field never lets the submitted value through — the account's owner sets
/// their password from the email.
fn inject_random_password(form: &crate::forms::Forms, data: &mut StrMap) {
    let typed_by_admin = form
        .fields
        .get("password")
        .is_some_and(|f| f.field_type() == "password" && !f.value().is_empty());
    if typed_by_admin {
        return;
    }
    let temp_pw = uuid::Uuid::new_v4().to_string();
    match crate::utils::password::hash(&temp_pw) {
        Ok(hash) => {
            data.insert("password".to_string(), hash);
        }
        // Never keep the submitted value in its place.
        Err(_) => {
            data.remove("password");
        }
    }
}

/// Replaces the parent-scope fields in a built form with hidden inputs carrying
/// the fixed values, so the picker (FK select / local-key widget) disappears and
/// the value is submitted as-is. `local_id` = `Some` on edit (pins the local key
/// of a composite child), `None` on create (the local key stays user-chosen).
fn hide_scope_fields(
    form: &mut Box<dyn crate::admin::helper::dyn_form::DynForm>,
    parent: &ParentBinding,
    local_id: Option<&str>,
) {
    use crate::forms::base::FormField;
    use crate::forms::fields::HiddenField;

    let mut fk = HiddenField::new(parent.fk_col);
    fk.set_value(&parent.parent_id);
    form.get_form_mut()
        .fields
        .insert(parent.fk_col.to_string(), Box::new(fk));

    if let (Some(col), Some(local)) = (parent.local_key, local_id) {
        let mut lk = HiddenField::new(col);
        lk.set_value(local);
        form.get_form_mut()
            .fields
            .insert(col.to_string(), Box::new(lk));
    }
}

pub(super) async fn handle_detail(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    state: &super::PrototypeAdminState,
    parent: Option<&ParentBinding>,
    current_user: &CurrentUser,
) -> AppResult<Response> {
    let closure_id = closure_id_of(parent, &id);
    let object = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), closure_id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?,
        None => None,
    };

    if let Some(mut v) = object {
        crate::admin::helper::resolve_fk_labels(
            &req.engine.db,
            std::slice::from_mut(&mut v),
            &entry.meta.fk_display,
        )
        .await;
        if let Some(apply_enum_labels) = entry.enum_label_fn {
            apply_enum_labels(&mut v);
        }
        format_datetime(&mut v);
        req.context.insert(ctx_detail::ENTRY, &v);
    }
    req.context.insert(ctx_detail::OBJECT_ID, &id);
    req.context.insert(
        "rich_fields",
        &*crate::utils::constante::parse::RICH_CONTENT_FIELDS,
    );
    if let Some(loader) = &entry.m2m_loader {
        let m2m_fields = loader(req.engine.db.clone(), Some(closure_id.clone())).await;
        req.context.insert("m2m_fields", &m2m_fields);
    }
    // Inline sub-lists: resources scoped as children of this one, filtered to it
    // (e.g. a group's rights on its detail page). Empty for resources with none.
    let inlines =
        super::handle_inline::build_inlines(req.engine.db.clone(), state, entry, &id, current_user)
            .await;
    req.context.insert("inlines", &inlines);
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
    parent: Option<&ParentBinding>,
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
    // Pre-fill the parent FK so a pre-selected picker (if any) is correct before
    // we hide it.
    let mut initial = StrMap::new();
    if let Some(p) = parent {
        force_scope_values(&mut initial, p, None);
    }
    let mut form = (entry.form_builder)(
        req.engine.db.clone(),
        resource_keys,
        initial,
        tera,
        csrf,
        axum::http::Method::GET,
    )
    .await;
    if let Some(p) = parent {
        hide_scope_fields(&mut form, p, None);
    }

    if let Some(loader) = &entry.m2m_loader {
        let m2m_fields = loader(req.engine.db.clone(), None).await;
        req.context.insert("m2m_fields", &m2m_fields);
    }
    inject_csp_nonce(&mut form, &req.context);
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
    parent: Option<&ParentBinding>,
) -> AppResult<Response> {
    // Trusted parent scope from the URL path — overrides any client-supplied FK.
    if let Some(p) = parent {
        force_scope_values(&mut body, p, None);
    }
    let raw_body = body.clone();
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
    if let Some(p) = parent {
        hide_scope_fields(&mut form, p, None);
    }
    let valid = form.is_valid().await;
    if let Some(level) = crate::utils::runique_log::get_log()
        .admin
        .as_ref()
        .and_then(|a| a.crud)
    {
        crate::runique_log!(level, resource = %entry.meta.key, valid, "create POST — form validation");
    }
    if valid {
        let mut body_for_create = accepted_data(form.get_form(), &raw_body);
        if let Some(p) = parent {
            force_scope_values(&mut body_for_create, p, None);
        }
        if entry.meta.inject_password {
            inject_random_password(form.get_form(), &mut body_for_create);
        }
        let result = match &entry.create_fn {
            Some(f) => f(req.engine.db.clone(), body_for_create.clone()).await,
            None => form.save(&req.engine.db).await,
        };
        match result {
            Ok(()) => {
                if let Some(level) = crate::utils::runique_log::get_log()
                    .admin
                    .as_ref()
                    .and_then(|a| a.crud)
                {
                    crate::runique_log!(level, resource = %entry.meta.key, "create POST — saved ok");
                }
            }
            Err(sea_orm::DbErr::Custom(ref msg)) => {
                if let Some(level) = crate::utils::runique_log::get_log()
                    .admin
                    .as_ref()
                    .and_then(|a| a.crud)
                {
                    crate::runique_log!(level, resource = %entry.meta.key, error = %msg, "create POST — custom DB error");
                }
                form.get_form_mut().errors.push(msg.clone());
                if let Some(loader) = &entry.m2m_loader {
                    let m2m_fields = loader(req.engine.db.clone(), None).await;
                    req.context.insert("m2m_fields", &m2m_fields);
                }
                inject_csp_nonce(&mut form, &req.context);
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
                if let Some(level) = crate::utils::runique_log::get_log()
                    .admin
                    .as_ref()
                    .and_then(|a| a.crud)
                {
                    crate::runique_log!(level, resource = %entry.meta.key, error = %e, unique = is_unique_violation(&e), "create POST — DB error");
                }
                form.get_form_mut().database_error(&e);
                if !is_unique_violation(&e) {
                    return Err(Box::new(AppError::new(ErrorContext::database(e))));
                }
                if let Some(loader) = &entry.m2m_loader {
                    let m2m_fields = loader(req.engine.db.clone(), None).await;
                    req.context.insert("m2m_fields", &m2m_fields);
                }
                inject_csp_nonce(&mut form, &req.context);
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

        let mut email_ok = true;
        if entry.meta.inject_password
            && let Some(email) = body_for_create.get("email")
        {
            let email_template = state
                .config
                .user_resources
                .get(entry.meta.key)
                .and_then(|t| t.as_deref());
            email_ok = super::handle_password::send_user_created_email(
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

        // send_user_created_email already pushed its own success/error notice
        // when the email step ran and failed — a blanket "created" notice on
        // top of it would misleadingly imply the whole operation succeeded.
        if email_ok {
            req.notices
                .success(t("admin.create.success").to_string())
                .await;
        }
        return Ok(Redirect::to(&format!(
            "{}/list",
            scope_base(&state.config.prefix, entry, parent)
        ))
        .into_response());
    }

    if let Some(loader) = &entry.m2m_loader {
        let m2m_fields = loader(req.engine.db.clone(), None).await;
        req.context.insert("m2m_fields", &m2m_fields);
    }
    inject_csp_nonce(&mut form, &req.context);
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
    parent: Option<&ParentBinding>,
) -> AppResult<Response> {
    let closure_id = closure_id_of(parent, &id);
    let tera = req.engine.tera.clone();
    let csrf = req
        .csrf_token
        .masked()
        .unwrap_or_else(|_| req.csrf_token.clone())
        .as_str()
        .to_string();

    let data = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), closure_id.clone())
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
    let mut form = (builder)(
        req.engine.db.clone(),
        resource_keys,
        data.clone(),
        tera,
        csrf,
        axum::http::Method::GET,
    )
    .await;
    if let Some(p) = parent {
        hide_scope_fields(&mut form, p, Some(&id));
    }

    if let Some(ts) = data.get("updated_at") {
        req.context.insert(ctx_edit::ORIG_UPDATED_AT, ts);
    }

    let return_qs = req
        .prisme
        .data
        .get(ctx_edit::RETURN_QS)
        .filter(|s| !s.is_empty())
        .cloned()
        .unwrap_or_default();

    if let Some(loader) = &entry.m2m_loader {
        let m2m_fields = loader(req.engine.db.clone(), Some(closure_id.clone())).await;
        req.context.insert("m2m_fields", &m2m_fields);
    }

    inject_csp_nonce(&mut form, &req.context);
    req.context.insert(ctx_edit::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_edit::IS_EDIT, &true);
    req.context.insert(ctx_edit::OBJECT_ID, &id);
    req.context.insert(ctx_edit::RETURN_QS, &return_qs);
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
    parent: Option<&ParentBinding>,
) -> AppResult<Response> {
    let closure_id = closure_id_of(parent, &id);
    let mut body_for_update = body.clone();
    let orig_updated_at = body_for_update.remove("__original_updated_at");
    let return_qs = body_for_update
        .remove("return_qs")
        .filter(|s| !s.is_empty());
    // Trusted parent scope from the URL path — the identity columns can't drift.
    if let Some(p) = parent {
        force_scope_values(&mut body_for_update, p, Some(&id));
    }

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
    if let Some(p) = parent {
        hide_scope_fields(&mut form, p, Some(&id));
    }

    let mut is_locked = false;
    let is_form_valid = form.is_valid().await;
    if let Some(level) = crate::utils::runique_log::get_log()
        .admin
        .as_ref()
        .and_then(|a| a.crud)
    {
        crate::runique_log!(level, resource = %entry.meta.key, id = %id, valid = is_form_valid, "edit POST — form validation");
    }

    let old_obj: Option<Value> = if is_form_valid {
        if let Some(get_fn) = &entry.get_fn {
            get_fn(req.engine.db.clone(), closure_id.clone())
                .await
                .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?
        } else {
            None
        }
    } else {
        None
    };

    // A malformed/unknown id must 404 here, before `update_fn`/`partial_update_fn`
    // are ever called on it — their own id-parse failure is a raw `DbErr`, not a
    // clean "not found". Only enforced when `get_fn` actually confirmed absence
    // (no `get_fn` configured means we can't tell, so don't block).
    if is_form_valid && entry.get_fn.is_some() && old_obj.is_none() {
        return Err(Box::new(AppError::new(ErrorContext::not_found(
            t("error.not_found").as_ref(),
        ))));
    }

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
        body_for_update = accepted_data(form.get_form(), &body_for_update);
        if let Some(p) = parent {
            force_scope_values(&mut body_for_update, p, Some(&id));
        }
        // Delete old files replaced by a new upload
        if let Some(ref old) = old_obj {
            let media_root = resolve_media_root();
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
                    if let Err(e) = std::fs::remove_file(&old_abs)
                        && e.kind() != std::io::ErrorKind::NotFound
                    {
                        tracing::warn!(path = %old_abs, error = %e, "old upload removal failed (edit)");
                    }
                }
            }
        }
        let summary = old_obj
            .as_ref()
            .and_then(|v| history::diff_fields(v, &body_for_update));
        // A custom edit_form_builder intentionally exposes a subset of the model's
        // fields (e.g. to inject admin-only actions without duplicating every column).
        // update_fn (admin_from_form) rebuilds the full ActiveModel from submitted data,
        // so fields missing from that narrower form get overwritten with their default —
        // partial_update_fn (Set only what's present, NotSet otherwise) avoids that.
        let result = match (
            entry.edit_form_builder.is_some(),
            &entry.partial_update_fn,
            &entry.update_fn,
        ) {
            (true, Some(f), _) => {
                f(req.engine.db.clone(), closure_id.clone(), body_for_update).await
            }
            (_, _, Some(f)) => f(req.engine.db.clone(), closure_id.clone(), body_for_update).await,
            _ => form.save(&req.engine.db).await,
        };
        if let Err(e) = result {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %entry.meta.key, id = %id, error = %e, unique = is_unique_violation(&e), "edit POST — DB error");
            }
            form.get_form_mut().database_error(&e);
            if !is_unique_violation(&e) {
                return Err(Box::new(AppError::new(ErrorContext::database(e))));
            }
        } else {
            if let Some(level) = crate::utils::runique_log::get_log()
                .admin
                .as_ref()
                .and_then(|a| a.crud)
            {
                crate::runique_log!(level, resource = %entry.meta.key, id = %id, "edit POST — saved ok");
            }
            if summary.is_some() {
                history::log_admin_action(
                    &req.engine.db,
                    history::AdminActionLog {
                        user_id: current_user.id,
                        username: &current_user.username,
                        resource_key: entry.meta.key,
                        object_pk: &closure_id,
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
            let base = scope_base(&state.config.prefix, entry, parent);
            let list_url = match return_qs {
                Some(qs) => format!("{}/list?{}", base, qs),
                None => format!("{}/list", base),
            };
            return Ok(Redirect::to(&list_url).into_response());
        }
    }

    if let Some(ts) = orig_updated_at {
        req.context.insert(ctx_edit::ORIG_UPDATED_AT, &ts);
    }

    if let Some(loader) = &entry.m2m_loader {
        let m2m_fields = loader(req.engine.db.clone(), Some(closure_id.clone())).await;
        req.context.insert("m2m_fields", &m2m_fields);
    }

    let return_qs_str = return_qs.as_deref().unwrap_or("");
    inject_csp_nonce(&mut form, &req.context);
    req.context.insert(ctx_edit::FORM_FIELDS, form.get_form());
    req.context.insert(ctx_edit::IS_EDIT, &true);
    req.context.insert(ctx_edit::OBJECT_ID, &id);
    req.context.insert(ctx_edit::RETURN_QS, return_qs_str);
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
    parent: Option<&ParentBinding>,
) -> AppResult<Response> {
    let closure_id = closure_id_of(parent, &id);
    let object = match &entry.get_fn {
        Some(f) => f(req.engine.db.clone(), closure_id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?,
        None => None,
    };

    if let Some(mut v) = object {
        crate::admin::helper::resolve_fk_labels(
            &req.engine.db,
            std::slice::from_mut(&mut v),
            &entry.meta.fk_display,
        )
        .await;
        if let Some(apply_enum_labels) = entry.enum_label_fn {
            apply_enum_labels(&mut v);
        }
        req.context.insert(ctx_detail::ENTRY, &v);
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
    parent: Option<&ParentBinding>,
) -> AppResult<Response> {
    let closure_id = closure_id_of(parent, &id);
    let delete_fn = entry.delete_fn.as_ref().ok_or_else(|| {
        Box::new(AppError::new(ErrorContext::not_found(
            t("admin.delete.not_found").as_ref(),
        )))
    })?;

    // Confirm existence before deleting — a malformed/unknown id must 404, not
    // reach `delete_fn` (whose own id-parse failure is a raw `DbErr`, not a
    // clean "not found").
    if let Some(get_fn) = &entry.get_fn {
        let exists = get_fn(req.engine.db.clone(), closure_id.clone())
            .await
            .map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?
            .is_some();
        if !exists {
            return Err(Box::new(AppError::new(ErrorContext::not_found(
                t("admin.delete.not_found").as_ref(),
            ))));
        }
    }

    let delete_result = delete_fn(req.engine.db.clone(), closure_id.clone()).await;
    if let Some(level) = crate::utils::runique_log::get_log()
        .admin
        .as_ref()
        .and_then(|a| a.crud)
    {
        match &delete_result {
            Ok(()) => {
                crate::runique_log!(level, resource = %entry.meta.key, id = %id, "delete POST — ok")
            }
            Err(e) => {
                crate::runique_log!(level, resource = %entry.meta.key, id = %id, error = %e, "delete POST — DB error")
            }
        }
    }
    delete_result.map_err(|e| Box::new(AppError::new(ErrorContext::database(e))))?;

    history::log_admin_action(
        &req.engine.db,
        history::AdminActionLog {
            user_id: current_user.id,
            username: &current_user.username,
            resource_key: entry.meta.key,
            object_pk: &closure_id,
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
    let base = scope_base(&state.config.prefix, entry, parent);
    let list_url = match return_qs {
        Some(qs) => format!("{}/list?{}", base, qs),
        None => format!("{}/list", base),
    };
    Ok(Redirect::to(&list_url).into_response())
}

#[cfg(test)]
mod tests {
    use super::{accepted_data, inject_random_password};
    use crate::forms::Forms;
    use crate::forms::fields::{HiddenField, TextField};
    use crate::utils::aliases::StrMap;

    fn body(pairs: &[(&str, &str)]) -> StrMap {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn accepted_data_keeps_form_fields_and_m2m_only() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::text("username"));
        form.fields
            .get_mut("username")
            .expect("field")
            .set_value("alice");
        let raw = body(&[
            ("username", "tampered"),
            ("is_active", "true"),
            ("m2m_tags__3", "on"),
        ]);
        let data = accepted_data(&form, &raw);
        assert_eq!(data.get("username").map(String::as_str), Some("alice"));
        assert!(!data.contains_key("is_active"), "not a form field");
        assert!(
            data.contains_key("m2m_tags__3"),
            "m2m checkboxes live outside the form"
        );
    }

    #[test]
    fn hidden_password_is_always_replaced() {
        let mut form = Forms::new("csrf");
        form.field(&HiddenField::new("password"));
        let mut data = body(&[("password", "$argon2id$chosen-by-the-client")]);
        inject_random_password(&form, &mut data);
        let stored = data.get("password").expect("password set");
        assert_ne!(stored, "$argon2id$chosen-by-the-client");
        assert!(stored.starts_with("$argon2"));
    }

    #[test]
    fn typed_password_is_kept() {
        let mut form = Forms::new("csrf");
        form.field(&TextField::password("password"));
        form.fields
            .get_mut("password")
            .expect("field")
            .set_value("$argon2id$hashed-by-finalize");
        let mut data = body(&[("password", "$argon2id$hashed-by-finalize")]);
        inject_random_password(&form, &mut data);
        assert_eq!(
            data.get("password").map(String::as_str),
            Some("$argon2id$hashed-by-finalize")
        );
    }
}
