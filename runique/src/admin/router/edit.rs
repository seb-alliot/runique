//! `GET/POST {admin}/{resource}/{id}/edit`: one row's form, written through the gate.
use super::context::{EditContext, FormTexts, ResourceRef, put};
use super::{chrome, detail_url, failure, unknown_resource};
use crate::admin::builder::registered::FormOutcome;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::admin::trad::insert_admin_messages;
use crate::auth::current_user::CurrentUser;
use crate::context::template::Request;
use crate::utils::aliases::AppResult;
use crate::utils::trad::t;
use axum::Extension;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect, Response};
use std::sync::Arc;

pub(crate) async fn edit_view(
    Path((resource, id)): Path<(String, String)>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let form = match model.edit(&req, &user, &id).await {
        Ok(FormOutcome::Show(form)) => form,
        Ok(FormOutcome::Saved(_)) => {
            req.notices.success(t("admin.edit.success")).await;
            let detail = detail_url(&req, &site, model.key(), &id);
            return Ok(Redirect::to(&detail).into_response());
        }
        Err(error) => return failure(error, &req, &admin).await,
    };
    let key = model.key();
    let cancel_url = detail_url(&req, &site, key, &id);

    chrome(&mut req, &admin, &site, &user, key);
    insert_admin_messages(&mut req.context, "edit");
    put(
        &mut req,
        &EditContext {
            resource: ResourceRef::new(key, model.title()),
            resource_key: key,
            current_page: "edit",
            texts: FormTexts {
                cancel_url,
                cancel_label: t("admin.edit.btn_cancel").into_owned(),
                submit_label: t("admin.edit.btn_submit").into_owned(),
                no_fields_label: t("admin.edit.no_fields").into_owned(),
            },
            object_id: id,
            form_fields: form,
        },
    )?;
    req.render(admin.config.templates.edit.resolve())
}
