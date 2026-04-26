use std::sync::Arc;

use axum::http::Method;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, PaginatorTrait, QuerySelect};

use crate::admin::{
    forms::DroitAdminForm,
    helper::{
        dyn_form::DynForm,
        resource_entry::{
            CountFn, CreateFn, DeleteFn, FormBuilder, GetFn, ListFn, ListParams, ResourceEntry,
            SortDir, UpdateFn,
        },
    },
    resource::AdminResource,
};
use crate::forms::field::RuniqueForm;
use crate::utils::{
    aliases::{ADb, ATera, StrMap},
    constante::{
        admin_context::{
            common::RESOURCE_KEY,
            permission::{
                CAN_CREATE, CAN_DELETE, CAN_DELETE_OWN, CAN_READ, CAN_UPDATE, CAN_UPDATE_OWN,
                GROUPE_ID, GROUPES,
            },
        },
        session_key::session::SESSION_USER_DROITS_KEY,
    },
    forms::parse_bool,
    trad::{t, tf},
};

pub(super) fn encode_droit_id(groupe_id: crate::utils::pk::Pk, resource_key: &str) -> String {
    format!("{}:{}", groupe_id, resource_key)
}

fn decode_droit_id(id: &str) -> Result<(crate::utils::pk::Pk, String), sea_orm::DbErr> {
    let mut parts = id.splitn(2, ':');
    let groupe_id = parts
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| sea_orm::DbErr::Custom(t("admin.builtin.invalid_id").into_owned()))?;
    let resource_key = parts
        .next()
        .ok_or_else(|| sea_orm::DbErr::Custom(t("admin.builtin.invalid_id").into_owned()))?
        .to_string();
    Ok((groupe_id, resource_key))
}

pub(super) fn droit_entry() -> ResourceEntry {
    use crate::admin::permissions::groupes_droits;

    let meta = AdminResource::new(
        SESSION_USER_DROITS_KEY,
        "runique::admin::permissions::groupes_droits::Model",
        "DroitAdminForm",
        SESSION_USER_DROITS_KEY,
        vec!["admin".to_string()],
    );

    let form_builder: FormBuilder = Arc::new(
        |db: ADb,
         _vec: Vec<std::string::String>,
         data: StrMap,
         tera: ATera,
         csrf: String,
         method: Method| {
            Box::pin(async move {
                let mut form = DroitAdminForm::build_with_data(&data, tera, &csrf, method).await;

                let submitted_groupe = data.get(GROUPE_ID).cloned().unwrap_or_default();
                let groupes = crate::admin::permissions::groupe::Entity::find()
                    .all(&*db)
                    .await
                    .unwrap_or_default();
                let groupe_choices = groupes
                    .into_iter()
                    .map(|g| crate::forms::fields::ChoiceOption::new(&g.id.to_string(), &g.nom))
                    .collect::<Vec<_>>();
                let mut gf = crate::forms::fields::ChoiceField::new(GROUPE_ID)
                    .choices(groupe_choices)
                    .label(t("admin.group").as_ref())
                    .required();
                {
                    use crate::forms::base::FormField;
                    if !submitted_groupe.is_empty() {
                        gf.set_value(&submitted_groupe);
                    }
                }
                form.get_form_mut()
                    .fields
                    .insert(GROUPE_ID.to_string(), Box::new(gf));

                let submitted_keys = data.get(RESOURCE_KEY).cloned().unwrap_or_default();
                let resource_choices = _vec
                    .into_iter()
                    .map(|key| crate::forms::fields::ChoiceOption::new(&key, &key))
                    .collect::<Vec<_>>();
                let mut rf = crate::forms::fields::CheckboxField::new(RESOURCE_KEY)
                    .choices(resource_choices)
                    .label(t("admin.resources").as_ref());
                {
                    use crate::forms::base::FormField;
                    if !submitted_keys.is_empty() {
                        rf.set_value(&submitted_keys);
                    }
                }
                form.get_form_mut()
                    .fields
                    .insert(RESOURCE_KEY.to_string(), Box::new(rf));

                Box::new(DroitDynWrapper(form)) as Box<dyn DynForm>
            })
        },
    );

    let edit_form_builder: FormBuilder = Arc::new(
        |db: ADb,
         _vec: Vec<std::string::String>,
         data: StrMap,
         tera: ATera,
         csrf: String,
         method: Method| {
            Box::pin(async move {
                let mut form = DroitAdminForm::build_with_data(&data, tera, &csrf, method).await;

                let current_groupe = data.get(GROUPE_ID).cloned().unwrap_or_default();
                let groupes = crate::admin::permissions::groupe::Entity::find()
                    .all(&*db)
                    .await
                    .unwrap_or_default();
                let groupe_choices = groupes
                    .into_iter()
                    .map(|g| {
                        let id_str = g.id.to_string();
                        let mut opt = crate::forms::fields::ChoiceOption::new(&id_str, &g.nom);
                        if id_str == current_groupe {
                            opt = opt.selected();
                        }
                        opt
                    })
                    .collect::<Vec<_>>();
                let mut gf = crate::forms::fields::ChoiceField::new(GROUPE_ID)
                    .choices(groupe_choices)
                    .label(t("admin.group").as_ref())
                    .required();
                {
                    use crate::forms::base::FormField;
                    gf.set_value(&current_groupe);
                }
                form.get_form_mut()
                    .fields
                    .insert(GROUPE_ID.to_string(), Box::new(gf));

                let current_key = data.get(RESOURCE_KEY).cloned().unwrap_or_default();
                let resource_choices = _vec
                    .into_iter()
                    .map(|key| {
                        let mut opt = crate::forms::fields::ChoiceOption::new(&key, &key);
                        if key == current_key {
                            opt = opt.selected();
                        }
                        opt
                    })
                    .collect::<Vec<_>>();
                let mut rf = crate::forms::fields::CheckboxField::new(RESOURCE_KEY)
                    .choices(resource_choices)
                    .label(t("admin.resource").as_ref());
                {
                    use crate::forms::base::FormField;
                    rf.set_value(&current_key);
                }
                form.get_form_mut()
                    .fields
                    .insert(RESOURCE_KEY.to_string(), Box::new(rf));

                Box::new(DroitDynWrapper(form)) as Box<dyn DynForm>
            })
        },
    );

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use crate::admin::permissions::groupe;
            use sea_orm::ColumnTrait;
            use sea_orm::{
                QueryFilter, QueryOrder,
                sea_query::{Alias, Expr, Order},
            };
            use std::collections::HashMap as HMap;

            let mut query = groupes_droits::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc {
                    Order::Desc
                } else {
                    Order::Asc
                };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            if let Some(ref search_str) = params.search {
                let escaped = search_str.replace('\'', "''");
                let mut cond = sea_orm::Condition::any();
                cond = cond.add(Expr::cust(format!(
                    "LOWER(CAST(resource_key AS TEXT)) LIKE LOWER('%{escaped}%')"
                )));
                query = query.filter(cond);
            }
            let rows = query
                .offset(params.offset)
                .limit(params.limit)
                .all(&*db)
                .await?;

            let groupe_ids: Vec<crate::utils::pk::Pk> = rows.iter().map(|r| r.groupe_id).collect();
            let groupes: HMap<crate::utils::pk::Pk, String> = groupe::Entity::find()
                .filter(groupe::Column::Id.is_in(groupe_ids))
                .all(&*db)
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|g| (g.id, g.nom))
                .collect();

            Ok(rows
                .into_iter()
                .map(|r| {
                    let mut v = serde_json::to_value(&r).unwrap_or(serde_json::Value::Null);
                    if let serde_json::Value::Object(ref mut map) = v {
                        let id_str = encode_droit_id(r.groupe_id, &r.resource_key);
                        map.insert("id".to_string(), serde_json::Value::String(id_str));
                        let nom = groupes.get(&r.groupe_id).cloned().unwrap_or_default();
                        map.insert(GROUPES.to_string(), serde_json::Value::String(nom));
                    }
                    v
                })
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::Expr};
            let mut query = groupes_droits::Entity::find();
            if let Some(ref search_str) = _search {
                let escaped = search_str.replace('\'', "''");
                query = query.filter(Expr::cust(format!(
                    "LOWER(CAST(resource_key AS TEXT)) LIKE LOWER('%{escaped}%')"
                )));
            }
            query.count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            use sea_orm::{ColumnTrait, QueryFilter};
            let (groupe_id, resource_key) = decode_droit_id(&id)?;
            let row = groupes_droits::Entity::find()
                .filter(groupes_droits::Column::GroupeId.eq(groupe_id))
                .filter(groupes_droits::Column::ResourceKey.eq(&resource_key))
                .one(&*db)
                .await?;
            let Some(row) = row else { return Ok(None) };
            let mut value = serde_json::to_value(&row).unwrap_or(serde_json::Value::Null);
            if let serde_json::Value::Object(ref mut map) = value {
                map.insert("id".to_string(), serde_json::Value::String(id));
            }
            Ok(Some(value))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            use sea_orm::{ColumnTrait, QueryFilter};
            let (groupe_id, resource_key) = decode_droit_id(&id)?;
            let result = groupes_droits::Entity::delete_many()
                .filter(groupes_droits::Column::GroupeId.eq(groupe_id))
                .filter(groupes_droits::Column::ResourceKey.eq(resource_key))
                .exec(&*db)
                .await
                .map(|_| ());
            if result.is_ok() {
                crate::auth::guard::clear_cache();
            }
            result
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            let groupe_id: crate::utils::pk::Pk = data
                .get(GROUPE_ID)
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| {
                    sea_orm::DbErr::Custom(t("admin.builtin.invalid_id").into_owned())
                })?;

            let resource_keys_str = data.get(RESOURCE_KEY).cloned().unwrap_or_default();
            let resource_keys: Vec<&str> = resource_keys_str
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();

            if resource_keys.is_empty() {
                return Err(sea_orm::DbErr::Custom(
                    t("admin.builtin.droit_resource_required").into_owned(),
                ));
            }

            for resource_key in &resource_keys {
                groupes_droits::ActiveModel {
                    groupe_id: Set(groupe_id),
                    resource_key: Set(resource_key.to_string()),
                    can_create: Set(parse_bool(&data, CAN_CREATE)),
                    can_read: Set(parse_bool(&data, CAN_READ)),
                    can_update: Set(parse_bool(&data, CAN_UPDATE)),
                    can_delete: Set(parse_bool(&data, CAN_DELETE)),
                    can_update_own: Set(parse_bool(&data, CAN_UPDATE_OWN)),
                    can_delete_own: Set(parse_bool(&data, CAN_DELETE_OWN)),
                }
                .insert(&*db)
                .await
                .map_err(|e| {
                    if super::is_unique_violation(&e) {
                        sea_orm::DbErr::Custom(tf("admin.builtin.droit_exists", &[resource_key]))
                    } else {
                        e
                    }
                })?;
            }
            crate::auth::guard::clear_cache();
            Ok(())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            use sea_orm::{ColumnTrait, QueryFilter};
            let (old_groupe_id, old_resource_key) = decode_droit_id(&id)?;

            let new_groupe_id: crate::utils::pk::Pk = data
                .get(GROUPE_ID)
                .and_then(|s| s.parse().ok())
                .unwrap_or(old_groupe_id);
            let new_resource_key = data
                .get(RESOURCE_KEY)
                .and_then(|s| s.split(',').find(|k| !k.trim().is_empty()))
                .unwrap_or(&old_resource_key)
                .trim()
                .to_string();

            groupes_droits::Entity::delete_many()
                .filter(groupes_droits::Column::GroupeId.eq(old_groupe_id))
                .filter(groupes_droits::Column::ResourceKey.eq(&old_resource_key))
                .exec(&*db)
                .await?;

            groupes_droits::ActiveModel {
                groupe_id: Set(new_groupe_id),
                resource_key: Set(new_resource_key),
                can_create: Set(parse_bool(&data, CAN_CREATE)),
                can_read: Set(parse_bool(&data, CAN_READ)),
                can_update: Set(parse_bool(&data, CAN_UPDATE)),
                can_delete: Set(parse_bool(&data, CAN_DELETE)),
                can_update_own: Set(parse_bool(&data, CAN_UPDATE_OWN)),
                can_delete_own: Set(parse_bool(&data, CAN_DELETE_OWN)),
            }
            .insert(&*db)
            .await
            .map_err(|e| {
                if super::is_unique_violation(&e) {
                    sea_orm::DbErr::Custom(t("admin.builtin.droit_exists").into_owned())
                } else {
                    e
                }
            })?;

            crate::auth::guard::clear_cache();
            Ok(())
        })
    });

    ResourceEntry::new(meta, form_builder)
        .with_edit_form_builder(edit_form_builder)
        .with_list_fn(list_fn)
        .with_count_fn(count_fn)
        .with_get_fn(get_fn)
        .with_delete_fn(delete_fn)
        .with_create_fn(create_fn)
        .with_update_fn(update_fn)
}

struct DroitDynWrapper(pub DroitAdminForm);

#[async_trait::async_trait]
impl DynForm for DroitDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }
    async fn save(&mut self, _db: &sea_orm::DatabaseConnection) -> Result<(), sea_orm::DbErr> {
        Ok(())
    }
    fn get_form(&self) -> &crate::forms::form::Forms {
        self.0.get_form()
    }
    fn get_form_mut(&mut self) -> &mut crate::forms::form::Forms {
        self.0.get_form_mut()
    }
}
