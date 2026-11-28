//! `GET {admin}/{resource}/{id}/detail`: one row of a model.
use crate::{
    admin::{
        builder::{registered::AdminError, site::AdminSite},
        router::{
            AdminView,
            account::{
                memberships::group_box,
                summary::{account_summaries, rank_key},
            },
            chrome,
            context::{DetailContext, ResourceRef, RowLinks, put},
            failure,
            routes::AdminState,
            unknown_resource,
        },
        template::Page,
        translation::insert_admin_messages,
    },
    auth::{
        constants::resource_key::GROUPES,
        current_user::CurrentUser,
        gate::{
            membership::{managed_groups, may_set_memberships},
            rights_grid::{may_set, may_set_group},
        },
        permissions::{group::Entity, rank::Rang},
        user::BuiltinUserEntity,
    },
    http::request::Request,
    schema::pk::Pk,
    utils::aliases::AppResult,
};
use axum::{Extension, extract::Path, response::Response};
use sea_orm::DbErr;
use serde_json::Value;
use std::sync::Arc;

/// `order` cut to the columns the row holds: a column the account can't read
/// isn't in it. A template's map loses its keys' order, hence the separate list.
fn readable_in_order(order: Vec<&'static str>, entry: &Value) -> Vec<&'static str> {
    order
        .into_iter()
        .filter(|column| entry.get(column).is_some())
        .collect()
}

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
    let detail_columns = readable_in_order(model.detail_order(), &entry);
    let key = model.key();
    let rights = model.rights(&user);
    let paths = &admin.paths;
    // The rank of an account below the viewer's, offered to change.
    let mut rank_choices = Vec::new();
    let mut current_rank = None;
    let mut rights_url = None;
    let mut memberships = Vec::new();
    let mut account_summary = None;
    if model.is_account_table()
        && let Ok(pk) = id.parse::<Pk>()
        && let Some(account) = BuiltinUserEntity::find_by_id(&req.engine.db, pk).await
    {
        let rang = Rang::from(&account);
        account_summary = match account_summaries(&req.engine.db, &[pk]).await {
            Ok(mut summaries) => summaries.remove(&pk),
            Err(e) => {
                return failure(AdminError::Db(e), &req, &admin).await;
            }
        };
        // The groups the viewer manages, ticked when the account belongs.
        if may_set_memberships(&user, &account) {
            use crate::auth::permissions::user_groups;
            use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
            let db = &req.engine.db;
            let groups = match managed_groups(&user, db).await {
                Ok(groups) => groups,
                Err(e) => {
                    return failure(AdminError::Db(e), &req, &admin).await;
                }
            };
            let member_of: Vec<i32> = match user_groups::Entity::find()
                .filter(user_groups::Column::UserId.eq(account.id))
                .all(db)
                .await
            {
                Ok(rows) => rows.into_iter().map(|m| m.groupe_id).collect(),
                Err(e) => {
                    return failure(AdminError::Db(e), &req, &admin).await;
                }
            };
            memberships = groups
                .into_iter()
                .map(|g| (group_box(g.id), g.nom, member_of.contains(&g.id)))
                .collect();
        }
        if may_set(&user, &account).is_ok() {
            rights_url = Some(paths.rights_of(key, &id));
        }
        if rang < user.rang && account.id != user.id {
            rank_choices = crate::admin::router::permission::rank::rank_choices(user.rang);
            current_rank = Some(rank_key(rang));
        }
    }

    // A group: its rights grid for whoever may set it, its transfer for a
    // dirigeant, and who took it back if it waits for a responsable.
    let mut transfer_receivers = Vec::new();
    let mut taken_by = None;
    if key == GROUPES
        && let Ok(group_id) = id.parse::<i32>()
        && let Ok(Some(group)) = <Entity as sea_orm::EntityTrait>::find_by_id(group_id)
            .one(&req.engine.db)
            .await
    {
        use crate::auth::gate::transfer;
        if may_set_group(&user, &group).is_ok() {
            rights_url = Some(paths.rights_of(key, &id));
        }
        let db = &req.engine.db;
        let found = async {
            let taken = transfer::taken_by(&[group_id], db).await?;
            let receivers = if user.rang >= Rang::Dirigeant {
                transfer::receivers(&group, db).await?
            } else {
                Vec::new()
            };
            Ok::<_, DbErr>((taken, receivers))
        };
        match found.await {
            Ok((taken, receivers)) => {
                taken_by = taken.get(&group_id).cloned();
                transfer_receivers = receivers
                    .into_iter()
                    .map(|(id, name)| (id.to_string(), name))
                    .collect();
            }
            Err(e) => {
                return failure(AdminError::Db(e), &req, &admin).await;
            }
        }
    }

    chrome(&mut req, &admin, &site, &user, key, Page::Detail);
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
            memberships_url: (!memberships.is_empty()).then(|| paths.groups_of(key, &id)),
            memberships,
            transfer_url: (!transfer_receivers.is_empty()).then(|| paths.transfer_of(key, &id)),
            transfer_receivers,
            taken_by,
            account: account_summary,
            object_id: id,
            detail_columns,
            entry,
            rich_fields: Vec::new(),
        },
    )?;
    req.render(
        admin
            .config
            .templates
            .page_for(Page::Detail, Some(model.templates())),
    )
}

#[cfg(test)]
mod tests {
    use super::readable_in_order;
    use serde_json::json;

    #[test]
    fn the_order_is_the_declared_one_not_the_rows() {
        let entry = json!({ "b": 1, "a": 2, "c": 3 });
        assert_eq!(
            readable_in_order(vec!["c", "a", "b"], &entry),
            ["c", "a", "b"]
        );
    }

    #[test]
    fn a_column_out_of_the_row_is_left_out() {
        let entry = json!({ "a": 1, "empty": null });
        assert_eq!(
            readable_in_order(vec!["hidden", "empty", "a"], &entry),
            ["empty", "a"]
        );
    }
}
