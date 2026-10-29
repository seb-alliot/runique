//! `GET/POST {admin}/{resource}/{id}/delete`: the confirmation, then the
//! delete through the gate.
use crate::admin::builder::registered::FormOutcome;
use crate::admin::builder::site::AdminSite;
use crate::admin::router::admin_router::AdminState;
use crate::admin::router::context::{DeleteContext, ResourceRef, put};
use crate::admin::router::{chrome, failure, unknown_resource};
use crate::admin::trad::insert_admin_messages;
use crate::auth::current_user::CurrentUser;
use crate::context::template::Request;
use crate::utils::aliases::AppResult;
use crate::utils::trad::t;
use axum::Extension;
use axum::extract::Path;
use axum::response::{IntoResponse, Redirect, Response};
use std::sync::Arc;

pub(crate) async fn delete_view(
    Path((resource, id)): Path<(String, String)>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    let key = model.key();
    let form = match model.delete(&req, &user, &id).await {
        Ok(FormOutcome::Show(form)) => form,
        Ok(FormOutcome::Saved(_)) => {
            req.notices.success(t("admin.delete.success")).await;
            return Ok(Redirect::to(&admin.paths.list_of(key)).into_response());
        }
        Err(error) => return failure(error, &req, &admin).await,
    };
    let cancel_url = admin.paths.detail_of(&site, key, &id);

    chrome(
        &mut req,
        &admin,
        &site,
        &user,
        key,
        crate::admin::template::Page::Delete,
    );
    insert_admin_messages(&mut req.context, "delete");
    put(
        &mut req,
        &DeleteContext {
            resource: ResourceRef::new(key, model.title()),
            resource_key: key,
            current_page: "delete",
            list_url: admin.paths.list_of(key),
            form_action: admin.paths.delete_of(key, &id),
            object_id: id,
            form_fields: form,
            cancel_url,
        },
    )?;
    req.render(admin.config.templates.page_for(
        crate::admin::template::Page::Delete,
        Some(model.templates()),
    ))
}
