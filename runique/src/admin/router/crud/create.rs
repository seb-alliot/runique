//! `GET/POST {admin}/{resource}/create`: a new row's form, written through the gate.
use crate::admin::builder::registered::FormOutcome;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::account::reset_password::{AccountLink, send_account_link};
use crate::admin::router::admin_router::AdminState;
use crate::admin::router::context::{CreateContext, FormTexts, ResourceRef, put};
use crate::admin::router::{chrome, failure, unknown_resource};
use crate::admin::translation::insert_admin_messages;
use crate::auth::current_user::CurrentUser;
use crate::auth::user::BuiltinUserEntity;
use crate::http::request::Request;
use crate::utils::aliases::AppResult;
use crate::utils::translation::t;
use axum::Extension;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect, Response};
use std::sync::Arc;

pub(crate) async fn create_view(
    Path(resource): Path<String>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let key = model.key();
    let list = admin.paths.list_of(key);
    // The staff never holds a password: a created account is opened by the
    // link mailed to its owner. Without a complete mailer configuration that
    // link can't leave, so no account is created at all.
    if model.is_account_table() && !crate::utils::mailer_configured() {
        tracing::warn!("account not created: the mailer configuration is missing or incomplete");
        req.notices
            .error(t(
                runique_i18n::keys::admin::reset_password::error::NO_MAILER,
            ))
            .await;
        return Ok(Redirect::to(&list).into_response());
    }
    let form = match model.create(&req, &user).await {
        Ok(FormOutcome::Show(form)) => form,
        Ok(FormOutcome::Saved(account_email)) => {
            req.notices
                .success(t(runique_i18n::keys::admin::create::info::SUCCESS))
                .await;
            // A created account: its owner gets the link to choose a password.
            if let Some(email) = account_email {
                let account = BuiltinUserEntity::find_by_email(&req.engine.db, &email).await;
                let sent = match account {
                    Some(account) => {
                        send_account_link(&req, &admin, account.id, AccountLink::Welcome).await
                    }
                    None => Err(
                        t(runique_i18n::keys::admin::reset_password::error::UNAVAILABLE)
                            .into_owned(),
                    ),
                };
                match sent {
                    Ok(_) => {
                        req.notices
                            .success(t(runique_i18n::keys::admin::user_created::info::EMAIL_SENT))
                            .await
                    }
                    Err(message) => req.notices.warning(message).await,
                }
            }
            return Ok(Redirect::to(&list).into_response());
        }
        Err(error) => return failure(error, &req, &admin).await,
    };

    chrome(
        &mut req,
        &admin,
        &site,
        &user,
        key,
        crate::admin::template::Page::Create,
    );
    insert_admin_messages(&mut req.context, "create");
    put(
        &mut req,
        &CreateContext {
            resource: ResourceRef::new(key, model.title()),
            resource_key: key,
            list_url: list.clone(),
            current_page: "create",
            form_fields: form,
            texts: FormTexts {
                form_action: admin.paths.create_of(key),
                cancel_url: list,
                cancel_label: t(runique_i18n::keys::admin::create::label::BTN_CANCEL).into_owned(),
                submit_label: t(runique_i18n::keys::admin::create::label::BTN_SUBMIT).into_owned(),
                no_fields_label: t(runique_i18n::keys::admin::create::label::NO_FIELDS)
                    .into_owned(),
            },
        },
    )?;
    req.render(admin.config.templates.page_for(
        crate::admin::template::Page::Create,
        Some(model.templates()),
    ))
}
