//! `GET/POST {admin}/{resource}/create`: a new row's form, written through the gate.
use super::context::{CreateContext, FormTexts, ResourceRef, put};
use super::reset_password::{AccountLink, send_account_link};
use super::{chrome, failure, link, unknown_resource};
use crate::admin::builder::registered::FormOutcome;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::admin::trad::insert_admin_messages;
use crate::auth::current_user::CurrentUser;
use crate::auth::user::BuiltinUserEntity;
use crate::context::template::Request;
use crate::utils::aliases::AppResult;
use crate::utils::constante::admin_key::admin_route;
use crate::utils::trad::t;
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
    let list = link(&req, admin_route::LIST, &[("resource", key)]);
    let form = match model.create(&req, &user).await {
        Ok(FormOutcome::Show(form)) => form,
        Ok(FormOutcome::Saved(account_email)) => {
            req.notices.success(t("admin.create.success")).await;
            // A created account: its owner gets the link to choose a password.
            if let Some(email) = account_email {
                let account = BuiltinUserEntity::find_by_email(&req.engine.db, &email).await;
                let sent = match account {
                    Some(account) => {
                        send_account_link(&req, &admin, account.id, AccountLink::Welcome).await
                    }
                    None => Err(t("admin.reset_password.error_unavailable").into_owned()),
                };
                match sent {
                    Ok(_) => {
                        req.notices
                            .success(t("admin.user_created.email_sent"))
                            .await
                    }
                    Err(message) => req.notices.warning(message).await,
                }
            }
            return Ok(Redirect::to(&list).into_response());
        }
        Err(error) => return failure(error, &req, &admin).await,
    };

    chrome(&mut req, &admin, &site, &user, key);
    insert_admin_messages(&mut req.context, "create");
    put(
        &mut req,
        &CreateContext {
            resource: ResourceRef::new(key, model.title()),
            resource_key: key,
            current_page: "create",
            form_fields: form,
            texts: FormTexts {
                cancel_url: list,
                cancel_label: t("admin.create.btn_cancel").into_owned(),
                submit_label: t("admin.create.btn_submit").into_owned(),
                no_fields_label: t("admin.create.no_fields").into_owned(),
            },
        },
    )?;
    req.render(admin.config.templates.create.resolve())
}
