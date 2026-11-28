//! `GET/POST {admin}/{resource}/{id}/edit`: one row's form, written through the gate.
use crate::{
    admin::{
        builder::{registered::FormOutcome, site::AdminSite},
        router::{
            chrome,
            context::{EditContext, FormTexts, ResourceRef, put},
            failure,
            routes::AdminState,
            unknown_resource,
        },
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
            req.notices
                .success(t(runique_i18n::keys::admin::edit::info::SUCCESS))
                .await;
            let detail = admin.paths.detail_of(&site, model.key(), &id);
            return Ok(Redirect::to(&detail).into_response());
        }
        Err(error) => return failure(error, &req, &admin).await,
    };
    let key = model.key();
    let cancel_url = admin.paths.detail_of(&site, key, &id);

    chrome(
        &mut req,
        &admin,
        &site,
        &user,
        key,
        crate::admin::template::Page::Edit,
    );
    insert_admin_messages(&mut req.context, "edit");
    put(
        &mut req,
        &EditContext {
            resource: ResourceRef::new(key, model.title()),
            resource_key: key,
            current_page: "edit",
            list_url: admin.paths.list_of(key),
            texts: FormTexts {
                form_action: admin.paths.edit_of(key, &id),
                cancel_url,
                cancel_label: t(runique_i18n::keys::admin::edit::label::BTN_CANCEL).into_owned(),
                submit_label: t(runique_i18n::keys::admin::edit::label::BTN_SUBMIT).into_owned(),
                no_fields_label: t(runique_i18n::keys::admin::edit::label::NO_FIELDS).into_owned(),
            },
            object_id: id,
            form_fields: form,
        },
    )?;
    req.render(
        admin
            .config
            .templates
            .page_for(crate::admin::template::Page::Edit, Some(model.templates())),
    )
}
