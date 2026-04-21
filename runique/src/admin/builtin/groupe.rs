use std::sync::Arc;

use axum::http::Method;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait, PaginatorTrait, QuerySelect};

use crate::admin::{
    forms::GroupeAdminForm,
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
    constante::admin_context::permission::GROUPES,
    trad::{t, tf},
};

pub(super) fn groupe_entry() -> ResourceEntry {
    use crate::admin::permissions::groupe;

    let meta = AdminResource::new(
        GROUPES,
        "runique::admin::permissions::groupe::Model",
        "GroupeAdminForm",
        GROUPES,
        vec!["admin".to_string()],
    );

    let form_builder: FormBuilder = Arc::new(
        |_db: ADb,
         _vec: Vec<std::string::String>,
         data: StrMap,
         tera: ATera,
         csrf: String,
         method: Method| {
            Box::pin(async move {
                let form = GroupeAdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(GroupeDynWrapper(form)) as Box<dyn DynForm>
            })
        },
    );

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter, QueryOrder,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = groupe::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc {
                    Order::Desc
                } else {
                    Order::Asc
                };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({col} AS TEXT) = '{escaped}'")));
            }
            if let Some(ref search_str) = params.search {
                let escaped = search_str.replace('\'', "''");
                let mut search_cond = sea_orm::Condition::any();
                let cols = vec!["id", "nom"];
                for col in cols {
                    search_cond = search_cond.add(Expr::cust(format!(
                        "LOWER(CAST({col} AS TEXT)) LIKE LOWER('%{escaped}%')"
                    )));
                }
                query = query.filter(search_cond);
            }
            let rows = query
                .offset(params.offset)
                .limit(params.limit)
                .all(&*db)
                .await?;
            Ok(rows
                .into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::Expr};
            let mut query = groupe::Entity::find();
            if let Some(ref search_str) = _search {
                let escaped = search_str.replace('\'', "''");
                let mut search_cond = sea_orm::Condition::any();
                let cols = vec!["id", "nom"];
                for col in cols {
                    search_cond = search_cond.add(Expr::cust(format!(
                        "LOWER(CAST({col} AS TEXT)) LIKE LOWER('%{escaped}%')"
                    )));
                }
                query = query.filter(search_cond);
            }
            query.count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<crate::utils::pk::Pk>()
                .map_err(|_| sea_orm::DbErr::Custom(t("admin.builtin.invalid_id").into_owned()))?;
            let row = groupe::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<crate::utils::pk::Pk>()
                .map_err(|_| sea_orm::DbErr::Custom(t("admin.builtin.invalid_id").into_owned()))?;
            let result = groupe::Entity::delete_by_id(id)
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
            let nom = data.get("nom").cloned().unwrap_or_default();
            groupe::ActiveModel {
                nom: Set(nom.clone()),
                ..Default::default()
            }
            .insert(&*db)
            .await
            .map(|_| ())
            .map_err(|e| {
                if super::is_unique_violation(&e) {
                    sea_orm::DbErr::Custom(tf("admin.builtin.groupe_exists", &[&nom]))
                } else {
                    e
                }
            })
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<crate::utils::pk::Pk>()
                .map_err(|_| sea_orm::DbErr::Custom(t("admin.builtin.invalid_id").into_owned()))?;
            let nom = data.get("nom").cloned().unwrap_or_default();
            let result = groupe::ActiveModel {
                id: Set(id),
                nom: Set(nom),
            }
            .update(&*db)
            .await
            .map(|_| ());
            if result.is_ok() {
                crate::auth::guard::clear_cache();
            }
            result
        })
    });

    ResourceEntry::new(meta, form_builder)
        .with_list_fn(list_fn)
        .with_count_fn(count_fn)
        .with_get_fn(get_fn)
        .with_delete_fn(delete_fn)
        .with_create_fn(create_fn)
        .with_update_fn(update_fn)
}

struct GroupeDynWrapper(pub GroupeAdminForm);

#[async_trait::async_trait]
impl DynForm for GroupeDynWrapper {
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
