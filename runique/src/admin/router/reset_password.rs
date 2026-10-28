//! `POST {admin}/{resource}/{id}/reset-password`: sends an account's owner a
//! link to choose a new password. The staff never sees it, nor any password.
use super::{detail_url, failure, unknown_resource};
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::auth::current_user::CurrentUser;
use crate::auth::history::{ActionLog, log_action};
use crate::auth::user::BuiltinUserEntity;
use crate::context::template::Request;
use crate::macros::routeur::register_url::reverse_with_parameters;
use crate::utils::aliases::AppResult;
use crate::utils::pk::Pk;
use crate::utils::trad::t;
use axum::Extension;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect, Response};
use std::sync::Arc;
use std::time::Duration;

/// How long the link works: what the email's text announces (one hour).
const RESET_LINK_TTL: Duration = Duration::from_secs(3600);

/// The page this app serves for `reset_password` (`with_password_reset`).
const RESET_ROUTE: &str = "reset_password";

/// Which email carries the link.
#[derive(Clone, Copy)]
pub(crate) enum AccountLink {
    /// "Reset your password", from the account's page.
    Reset,
    /// "Choose your password", to an account the staff just created.
    Welcome,
}

impl AccountLink {
    fn template(self, admin: &AdminState) -> &str {
        match self {
            Self::Reset => admin
                .config
                .reset_password_email_template
                .as_deref()
                .unwrap_or("admin/reset_password_email.html"),
            Self::Welcome => "admin/user_created_email.html",
        }
    }

    fn subject(self) -> &'static str {
        match self {
            Self::Reset => "admin.reset_password.email_subject",
            Self::Welcome => "admin.user_created.email_subject",
        }
    }

    /// The template's `t_*` texts and their translation keys.
    fn texts(self) -> &'static [(&'static str, &'static str)] {
        match self {
            Self::Reset => &[
                ("t_title", "admin.reset_password.email_title"),
                ("t_greeting", "admin.reset_password.email_greeting"),
                ("t_body", "admin.reset_password.email_body"),
                ("t_btn", "admin.reset_password.email_btn"),
                ("t_ignore", "admin.reset_password.email_ignore"),
            ],
            Self::Welcome => &[
                ("t_greeting", "admin.user_created.email_greeting"),
                ("t_body", "admin.user_created.email_body"),
                ("t_btn", "admin.user_created.email_btn"),
                ("t_validity", "admin.user_created.email_validity"),
            ],
        }
    }
}

/// Generates the token and mails `account_id`'s owner the link to the reset
/// page, then gives the account's username; the one message the staff sees
/// when it can't, whatever the reason (traced).
pub(crate) async fn send_account_link(
    req: &Request,
    admin: &AdminState,
    account_id: Pk,
    kind: AccountLink,
) -> Result<String, String> {
    let unavailable = || t("admin.reset_password.error_unavailable").into_owned();
    let db = &req.engine.db;
    let Some(account) = BuiltinUserEntity::find_by_id(db, account_id).await else {
        return Err(unavailable());
    };
    // Activated once, inactive now: the staff blocked it. No link, as on the
    // site's own "forgot password" page.
    if account.activated_at.is_some() && !account.is_active {
        return Err(unavailable());
    }
    // No mailer, no public URL or no reset page: the link could only be shown
    // to the staff, who must never hold it.
    let Some(host) = req.public_url() else {
        return Err(unavailable());
    };
    if !crate::utils::mailer_configured() {
        return Err(unavailable());
    }
    let token = crate::utils::reset_token::generate(db, account.id, RESET_LINK_TTL)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "admin reset token generation failed");
            unavailable()
        })?;
    let encrypted = crate::utils::reset_token::encrypt_email(&token, &account.email);
    let path = reverse_with_parameters(
        &req.engine,
        RESET_ROUTE,
        &[("token", &token), ("encrypted_email", &encrypted)],
    )
    .map_err(|_| unavailable())?;

    let mut ctx = tera::Context::new();
    ctx.insert("username", &account.username);
    ctx.insert("reset_url", &format!("{host}{path}"));
    for (key, text) in kind.texts() {
        ctx.insert(*key, t(text).as_ref());
    }
    crate::utils::Email::new()
        .to(account.email.clone())
        .subject(t(kind.subject()).to_string())
        .template(&req.engine.tera, kind.template(admin), ctx)
        .map_err(|e| {
            tracing::error!(error = %e, "admin reset email render failed");
            unavailable()
        })?
        .send()
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "admin reset email send failed");
            unavailable()
        })?;
    Ok(account.username)
}

pub(crate) async fn reset_password_view(
    Path((resource, id)): Path<(String, String)>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let detail = detail_url(&req, &site, model.key(), &id);
    let account = match model.reset_password(&req, &user, &id).await {
        Ok(Some(account)) => account,
        Ok(None) => {
            req.notices.error(t("csrf.invalid_or_missing")).await;
            return Ok(Redirect::to(&detail).into_response());
        }
        Err(error) => return failure(error, &req, &admin).await,
    };
    match send_account_link(&req, &admin, account, AccountLink::Reset).await {
        Ok(username) => {
            // The action is traced like a write: who sent a link to whom,
            // the account known by its username as in every other entry.
            if let Err(e) = log_action(
                &req.engine.db,
                ActionLog {
                    user_id: user.id,
                    username: &user.username,
                    resource_key: model.key(),
                    object_pk: &id,
                    object_repr: &username,
                    action: "reset_password",
                    summary: None,
                    batch_id: None,
                },
            )
            .await
            {
                tracing::error!(error = %e, "admin reset trace failed");
            }
            req.notices
                .success(t("admin.reset_password.success_email"))
                .await;
        }
        Err(message) => req.notices.error(message).await,
    }
    Ok(Redirect::to(&detail).into_response())
}
