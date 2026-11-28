//! `GET/POST {admin}/{resource}/{id}/delete`: the confirmation, then the
//! delete through the gate.
use crate::{
    admin::{
        builder::{registered::FormOutcome, site::AdminSite},
        router::{
            chrome,
            context::{DeleteContext, ResourceRef, put},
            failure,
            routes::AdminState,
            unknown_resource,
        },
        template::Page,
        translation::insert_admin_messages,
    },
    auth::current_user::CurrentUser,
    http::request::Request,
    translation::t,
    utils::aliases::AppResult,
};
use axum::{
    Extension,
    extract::Path,
    response::{IntoResponse, Redirect, Response},
};
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
            req.notices
                .success(t(runique_i18n::keys::admin::delete::info::SUCCESS))
                .await;
            return Ok(Redirect::to(&admin.paths.list_of(key)).into_response());
        }
        Err(error) => return failure(error, &req, &admin).await,
    };
    let cancel_url = admin.paths.detail_of(&site, key, &id);

    chrome(&mut req, &admin, &site, &user, key, Page::Delete);
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
    req.render(
        admin
            .config
            .templates
            .page_for(Page::Delete, Some(model.templates())),
    )
}
