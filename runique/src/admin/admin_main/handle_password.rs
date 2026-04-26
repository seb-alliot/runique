use crate::admin::helper::resource_entry::ResourceEntry;
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::{aliases::AppResult, trad::t};
use axum::response::{IntoResponse, Redirect, Response};

pub(super) async fn send_user_created_email(
    req: &mut Request,
    _entry: &ResourceEntry,
    email: &str,
    username: Option<&str>,
    email_template: Option<&str>,
    headers: &axum::http::HeaderMap,
    state: &super::PrototypeAdminState,
) {
    let token = crate::utils::reset_token::generate(email);
    let encrypted = crate::utils::reset_token::encrypt_email(&token, email);

    let reset_url = if let Some(base) = &state.config.reset_password_url {
        format!("{}/{}/{}", base.trim_end_matches('/'), token, encrypted)
    } else {
        let host = headers
            .get(axum::http::header::HOST)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("localhost");
        format!("http://{}/reset-password/{}/{}", host, token, encrypted)
    };

    let template_name = email_template.unwrap_or("admin/user_created_email.html");
    let username_str = username.unwrap_or(email);

    let mut ctx = tera::Context::new();
    ctx.insert("username", username_str);
    ctx.insert("email", email);
    ctx.insert("reset_url", &reset_url);
    ctx.insert(
        "t_greeting",
        t("admin.user_created.email_greeting").as_ref(),
    );
    ctx.insert("t_body", t("admin.user_created.email_body").as_ref());
    ctx.insert("t_btn", t("admin.user_created.email_btn").as_ref());
    ctx.insert(
        "t_validity",
        t("admin.user_created.email_validity").as_ref(),
    );

    let body_html = match req.engine.tera.render(template_name, &ctx) {
        Ok(rendered) => rendered,
        Err(_) => format!(
            "<p>Hello {username_str},</p><p>Click on the link to set your password:</p><p><a href=\"{reset_url}\">{reset_url}</a></p>"
        ),
    };

    if crate::utils::mailer_configured() {
        match crate::utils::Email::new()
            .to(email)
            .subject(t("admin.user_created.email_subject"))
            .html(body_html)
            .send()
            .await
        {
            Ok(_) => {
                req.notices
                    .success(crate::utils::trad::tf(
                        "admin.user_created.email_sent",
                        &[email],
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
}

pub(super) async fn handle_reset_password(
    req: &mut Request,
    entry: &ResourceEntry,
    id: String,
    headers: &axum::http::HeaderMap,
    state: &super::PrototypeAdminState,
) -> AppResult<Response> {
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

    if crate::utils::mailer_configured() {
        let template_name = state
            .config
            .reset_password_email_template
            .as_deref()
            .unwrap_or("admin/reset_password_email.html");
        let mut ctx = tera::Context::new();
        ctx.insert("username", username);
        ctx.insert("email", &email);
        ctx.insert("reset_url", &reset_url);
        ctx.insert("t_title", t("admin.reset_password.email_title").as_ref());
        ctx.insert(
            "t_greeting",
            t("admin.reset_password.email_greeting").as_ref(),
        );
        ctx.insert("t_body", t("admin.reset_password.email_body").as_ref());
        ctx.insert("t_btn", t("admin.reset_password.btn").as_ref());
        ctx.insert("t_ignore", t("admin.reset_password.email_ignore").as_ref());
        let body = match req.engine.tera.render(template_name, &ctx) {
            Ok(rendered) => rendered,
            Err(_) => format!(
                "<p>Hello {username},</p><p>Click on the following link to reset your password (valid for 1 hour):</p><p><a href=\"{reset_url}\">{reset_url}</a></p>"
            ),
        };
        match crate::utils::Email::new()
            .to(email.clone())
            .subject(t("admin.reset_password.email_subject"))
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
