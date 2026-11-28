//! `POST {admin}/{resource}/{id}/reset-password`: sends an account's owner a
//! link to choose a new password. The staff never sees it, nor any password.
use crate::{
    admin::{
        builder::site::AdminSite,
        router::{failure, routes::AdminState, unknown_resource},
    },
    auth::{
        current_user::CurrentUser,
        history::{ActionLog, log_action},
        password::{ResetMail, send_reset_link},
        user::BuiltinUserEntity,
    },
    http::request::Request,
    mailer::mailer_configured,
    schema::pk::Pk,
    translation::{current_lang, t, tf},
    utils::aliases::AppResult,
};
use axum::{
    Extension,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
use std::sync::Arc;

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
            Self::Reset => runique_i18n::keys::admin::reset_password::label::EMAIL_SUBJECT,
            Self::Welcome => runique_i18n::keys::admin::user_created::label::EMAIL_SUBJECT,
        }
    }

    /// The template's `t_*` texts, the link's lifetime in `minutes` filled in.
    fn texts(self, minutes: u64) -> Vec<(&'static str, String)> {
        let text = |key: &str| t(key).into_owned();
        let minutes = [minutes.to_string()];
        // The language the mail is written in, for its `<html lang>`.
        let lang = ("lang", current_lang().code().to_string());
        match self {
            Self::Reset => vec![
                lang,
                (
                    "t_title",
                    text(runique_i18n::keys::admin::reset_password::label::EMAIL_TITLE),
                ),
                (
                    "t_greeting",
                    text(runique_i18n::keys::admin::reset_password::label::EMAIL_GREETING),
                ),
                (
                    "t_body",
                    tf(
                        runique_i18n::keys::admin::reset_password::label::EMAIL_BODY,
                        &minutes,
                    ),
                ),
                (
                    "t_btn",
                    text(runique_i18n::keys::admin::reset_password::label::EMAIL_BTN),
                ),
                (
                    "t_ignore",
                    text(runique_i18n::keys::admin::reset_password::label::EMAIL_IGNORE),
                ),
            ],
            Self::Welcome => vec![
                lang,
                (
                    "t_greeting",
                    text(runique_i18n::keys::admin::user_created::label::EMAIL_GREETING),
                ),
                (
                    "t_body",
                    text(runique_i18n::keys::admin::user_created::label::EMAIL_BODY),
                ),
                (
                    "t_btn",
                    text(runique_i18n::keys::admin::user_created::label::EMAIL_BTN),
                ),
                (
                    "t_validity",
                    tf(
                        runique_i18n::keys::admin::user_created::label::EMAIL_VALIDITY,
                        &minutes,
                    ),
                ),
            ],
        }
    }
}

/// Mails `account_id`'s owner the link to the reset page, through the one
/// function that makes reset links, then gives the account's username; the
/// one message the staff sees when it can't, whatever the reason (traced).
pub(crate) async fn send_account_link(
    req: &Request,
    admin: &AdminState,
    account_id: Pk,
    kind: AccountLink,
) -> Result<String, String> {
    let unavailable =
        || t(runique_i18n::keys::admin::reset_password::error::UNAVAILABLE).into_owned();
    // Asked before anything else: no token is written for a link that can't leave.
    if !mailer_configured() {
        tracing::warn!("admin link not sent: the mailer configuration is missing or incomplete");
        return Err(t(runique_i18n::keys::admin::reset_password::error::NO_MAILER).into_owned());
    }
    let Some(account) = BuiltinUserEntity::find_by_id(&req.engine.db, account_id).await else {
        return Err(unavailable());
    };
    // No public URL or no reset page: the link could only be shown to the
    // staff, who must never hold it.
    let (Some(host), Some(link)) = (req.public_url(), admin.config.reset_link.as_ref()) else {
        return Err(unavailable());
    };
    let mail = ResetMail::Template {
        name: kind.template(admin).to_string(),
        subject: t(kind.subject()).to_string(),
        texts: kind.texts(link.minutes()),
    };
    match send_reset_link(&req.engine, &host, link, &account, mail).await {
        Ok(()) => Ok(account.username),
        Err(e) => {
            tracing::warn!(error = %e, "admin reset link not sent");
            Err(unavailable())
        }
    }
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
    let detail = admin.paths.detail_of(&site, model.key(), &id);
    let account = match model.reset_password(&req, &user, &id).await {
        Ok(Some(account)) => account,
        Ok(None) => {
            req.notices
                .error(t(runique_i18n::keys::csrf::error::INVALID_OR_MISSING))
                .await;
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
                .success(t(
                    runique_i18n::keys::admin::reset_password::info::SUCCESS_EMAIL,
                ))
                .await;
        }
        Err(message) => req.notices.error(message).await,
    }
    Ok(Redirect::to(&detail).into_response())
}
