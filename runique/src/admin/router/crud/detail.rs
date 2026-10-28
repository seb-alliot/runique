//! `GET {admin}/{resource}/{id}/detail`: one row of a model.
use crate::admin::builder::site::AdminSite;
use crate::admin::router::AdminView;
use crate::admin::router::admin_router::AdminState;
use crate::admin::router::context::{DetailContext, ResourceRef, RowLinks, put};
use crate::admin::router::{chrome, failure, unknown_resource};
use crate::admin::trad::insert_admin_messages;
use crate::auth::current_user::CurrentUser;
use crate::context::template::Request;
use crate::utils::aliases::AppResult;
use axum::Extension;
use axum::extract::Path;
use axum::response::Response;
use std::sync::Arc;

pub(crate) async fn detail_view(
    Path((resource, id)): Path<(String, String)>,
    Extension(site): Extension<Arc<AdminSite>>,
    Extension(admin): Extension<Arc<AdminState>>,
    Extension(user): Extension<CurrentUser>,
    mut req: Request,
) -> AppResult<Response> {
    let Some(model) = site.model(&resource) else {
        return unknown_resource(&resource);
    };
    if site.overridden_path(&resource, AdminView::Detail).is_some() {
        return unknown_resource(&resource);
    }
    let entry = match model.detail(&req.engine.db, &user, &id).await {
        Ok(entry) => entry,
        Err(error) => return failure(error, &req, &admin).await,
    };
    let key = model.key();
    let rights = model.rights(&user);
    let paths = &admin.paths;
    // The rank of an account below the viewer's, offered to change.
    let mut rank_choices = Vec::new();
    let mut current_rank = None;
    let mut rights_url = None;
    if model.is_account_table()
        && let Ok(pk) = id.parse::<crate::utils::pk::Pk>()
        && let Some(account) =
            crate::auth::user::BuiltinUserEntity::find_by_id(&req.engine.db, pk).await
    {
        let rang = crate::auth::permissions::rang::Rang::from(&account);
        if crate::auth::gate::rights_grid::may_set(&user, &account).is_ok() {
            rights_url = Some(paths.rights_of(key, &id));
        }
        if rang < user.rang && account.id != user.id {
            rank_choices = crate::admin::router::permission::rank::rank_choices(user.rang);
            current_rank = Some(match rang {
                crate::auth::permissions::rang::Rang::Lambda => "lambda",
                crate::auth::permissions::rang::Rang::Admin => "admin",
                crate::auth::permissions::rang::Rang::Dirigeant => "dirigeant",
                crate::auth::permissions::rang::Rang::Superuser => "superuser",
            });
        }
    }

    // A group's rights grid, for whoever may set it.
    if key == crate::utils::constante::resource_key::GROUPES
        && let Ok(group_id) = id.parse::<i32>()
        && let Ok(Some(group)) =
            <crate::auth::permissions::groupe::Entity as sea_orm::EntityTrait>::find_by_id(group_id)
                .one(&req.engine.db)
                .await
        && crate::auth::gate::rights_grid::may_set_group(&user, &group).is_ok()
    {
        rights_url = Some(paths.rights_of(key, &id));
    }

    chrome(&mut req, &admin, &site, &user, key);
    insert_admin_messages(&mut req.context, "detail");
    insert_admin_messages(&mut req.context, "list");
    put(
        &mut req,
        &DetailContext {
            resource: ResourceRef::new(key, model.title()),
            resource_key: key,
            current_page: "detail",
            list_url: paths.list_of(key),
            links: RowLinks {
                detail: paths.detail_of(&site, key, &id),
                edit: (rights.can_update || rights.can_update_own).then(|| paths.edit_of(key, &id)),
                delete: (rights.can_delete || rights.can_delete_own)
                    .then(|| paths.delete_of(key, &id)),
            },
            reset_password_url: (model.is_account_table() && rights.can_update)
                .then(|| paths.reset_password_of(key, &id)),
            rank_url: (!rank_choices.is_empty()).then(|| paths.rank_of(key)),
            rank_choices,
            current_rank,
            rights_url,
            object_id: id,
            entry,
            rich_fields: Vec::new(),
        },
    )?;
    req.render(admin.config.templates.detail.resolve())
}
