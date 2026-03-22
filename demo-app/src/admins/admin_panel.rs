// AUTO-admin_panel — DO NOT EDIT MANUALLY
// admin_panel by `runique start` from src/admin.rs

use runique::admin::resource_entry::FilterFn;
use runique::prelude::*;

use crate::entities::blog;
use crate::entities::changelog_entry;
use crate::entities::chapitre;
use crate::entities::code_example;
use crate::entities::cour;
use crate::entities::cour_block;
use crate::entities::demo_category;
use crate::entities::demo_page;
use crate::entities::demo_section;
use crate::entities::doc_block;
use crate::entities::doc_page;
use crate::entities::doc_section;
use crate::entities::form_field;
use crate::entities::known_issue;
use crate::entities::page_doc_link;
use crate::entities::roadmap_entry;
use crate::entities::site_config;
use crate::entities::users;

// ── DynForm wrapper pour users::AdminForm ──
struct UsersAdminFormDynWrapper(pub users::AdminForm);

#[async_trait]
impl DynForm for UsersAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm edit wrapper pour crate::formulaire::UserEditForm ──
struct UsersEditFormDynWrapper(pub crate::formulaire::UserEditForm);

#[async_trait]
impl DynForm for UsersEditFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, _db: &DatabaseConnection) -> Result<(), DbErr> {
        Ok(()) // update_fn gère la persistance
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour blog::AdminForm ──
struct BlogAdminFormDynWrapper(pub blog::AdminForm);

#[async_trait]
impl DynForm for BlogAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour changelog_entry::AdminForm ──
struct ChangelogEntryAdminFormDynWrapper(pub changelog_entry::AdminForm);

#[async_trait]
impl DynForm for ChangelogEntryAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour roadmap_entry::AdminForm ──
struct RoadmapEntryAdminFormDynWrapper(pub roadmap_entry::AdminForm);

#[async_trait]
impl DynForm for RoadmapEntryAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour known_issue::AdminForm ──
struct KnownIssueAdminFormDynWrapper(pub known_issue::AdminForm);

#[async_trait]
impl DynForm for KnownIssueAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour demo_category::AdminForm ──
struct DemoCategoryAdminFormDynWrapper(pub demo_category::AdminForm);

#[async_trait]
impl DynForm for DemoCategoryAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour demo_page::AdminForm ──
struct DemoPageAdminFormDynWrapper(pub demo_page::AdminForm);

#[async_trait]
impl DynForm for DemoPageAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour demo_section::AdminForm ──
struct DemoSectionAdminFormDynWrapper(pub demo_section::AdminForm);

#[async_trait]
impl DynForm for DemoSectionAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour code_example::AdminForm ──
struct CodeExampleAdminFormDynWrapper(pub code_example::AdminForm);

#[async_trait]
impl DynForm for CodeExampleAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour page_doc_link::AdminForm ──
struct PageDocLinkAdminFormDynWrapper(pub page_doc_link::AdminForm);

#[async_trait]
impl DynForm for PageDocLinkAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour form_field::AdminForm ──
struct FormFieldAdminFormDynWrapper(pub form_field::AdminForm);

#[async_trait]
impl DynForm for FormFieldAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour doc_section::AdminForm ──
struct DocSectionAdminFormDynWrapper(pub doc_section::AdminForm);

#[async_trait]
impl DynForm for DocSectionAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour doc_page::AdminForm ──
struct DocPageAdminFormDynWrapper(pub doc_page::AdminForm);

#[async_trait]
impl DynForm for DocPageAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour doc_block::AdminForm ──
struct DocBlockAdminFormDynWrapper(pub doc_block::AdminForm);

#[async_trait]
impl DynForm for DocBlockAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour site_config::AdminForm ──
struct SiteConfigAdminFormDynWrapper(pub site_config::AdminForm);

#[async_trait]
impl DynForm for SiteConfigAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour cour::AdminForm ──
struct CourAdminFormDynWrapper(pub cour::AdminForm);

#[async_trait]
impl DynForm for CourAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour chapitre::AdminForm ──
struct ChapitreAdminFormDynWrapper(pub chapitre::AdminForm);

#[async_trait]
impl DynForm for ChapitreAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

// ── DynForm wrapper pour cour_block::AdminForm ──
struct CourBlockAdminFormDynWrapper(pub cour_block::AdminForm);

#[async_trait]
impl DynForm for CourBlockAdminFormDynWrapper {
    async fn is_valid(&mut self) -> bool {
        self.0.is_valid().await
    }

    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        self.0.save(db).await
    }

    fn get_form(&self) -> &Forms {
        self.0.get_form()
    }

    fn get_form_mut(&mut self) -> &mut Forms {
        self.0.get_form_mut()
    }
}

/// Construit le registre admin au boot.
/// Appelé par le builder de l'application.
pub fn admin_register() -> AdminRegistry {
    runique::admin::register_roles(vec!["admin".to_string()]);
    let mut registry = AdminRegistry::new();

    // ── Ressource : users ──
    let meta = AdminResource::new(
        "users",
        "crate::entities::users::Model",
        "AdminForm",
        "Utilisateurs",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = users::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(UsersAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = users::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { users::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = users::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            users::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            users::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            users::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    let edit_form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    crate::formulaire::UserEditForm::build_with_data(&data, tera, &csrf, method)
                        .await;
                Box::new(UsersEditFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let meta = meta.display(
        DisplayConfig::new()
            .columns_include(vec![
                ("username", "Nom d'utilisateur"),
                ("email", "Email"),
                ("is_superuser", "Superuser"),
                ("is_active", "Actif"),
            ])
            .list_filter(vec![
                ("is_superuser", "Superuser", 10u64),
                ("is_active", "Actif", 10u64),
            ]),
    );
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::sea_query::{Alias, Expr, Order, Query};
            use sea_orm::{ConnectionTrait, ExprTrait};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> =
                std::collections::HashMap::new();
            let page_size_is_superuser = 10u64;
            let cur_page_is_superuser = pages.get("is_superuser").copied().unwrap_or(0);
            let count_stmt_is_superuser = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT is_superuser)"))
                .from(Alias::new(users::Entity.table_name()))
                .and_where(Expr::col(Alias::new("is_superuser")).is_not_null())
                .to_owned();
            let count_row_is_superuser =
                db.query_one(&count_stmt_is_superuser).await.unwrap_or(None);
            let total_is_superuser = count_row_is_superuser
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_is_superuser = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(is_superuser AS TEXT)"))
                .from(Alias::new(users::Entity.table_name()))
                .and_where(Expr::col(Alias::new("is_superuser")).is_not_null())
                .order_by(Alias::new("is_superuser"), Order::Asc)
                .limit(page_size_is_superuser)
                .offset(cur_page_is_superuser * page_size_is_superuser)
                .to_owned();
            let rows_is_superuser = db.query_all(&stmt_is_superuser).await.unwrap_or_default();
            let vals_is_superuser: Vec<String> = rows_is_superuser
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert(
                "is_superuser".to_string(),
                (vals_is_superuser, total_is_superuser),
            );
            let page_size_is_active = 10u64;
            let cur_page_is_active = pages.get("is_active").copied().unwrap_or(0);
            let count_stmt_is_active = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT is_active)"))
                .from(Alias::new(users::Entity.table_name()))
                .and_where(Expr::col(Alias::new("is_active")).is_not_null())
                .to_owned();
            let count_row_is_active = db.query_one(&count_stmt_is_active).await.unwrap_or(None);
            let total_is_active = count_row_is_active
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_is_active = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(is_active AS TEXT)"))
                .from(Alias::new(users::Entity.table_name()))
                .and_where(Expr::col(Alias::new("is_active")).is_not_null())
                .order_by(Alias::new("is_active"), Order::Asc)
                .limit(page_size_is_active)
                .offset(cur_page_is_active * page_size_is_active)
                .to_owned();
            let rows_is_active = db.query_all(&stmt_is_active).await.unwrap_or_default();
            let vals_is_active: Vec<String> = rows_is_active
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert("is_active".to_string(), (vals_is_active, total_is_active));
            Ok(result)
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_edit_form_builder(edit_form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn)
            .with_filter_fn(filter_fn),
    );

    // ── Ressource : blog ──
    let meta = AdminResource::new(
        "blog",
        "crate::entities::blog::Model",
        "AdminForm",
        "Articles",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = blog::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(BlogAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = blog::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { blog::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = blog::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            blog::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            blog::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            blog::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : changelog_entry ──
    let meta = AdminResource::new(
        "changelog_entry",
        "crate::entities::changelog_entry::Model",
        "AdminForm",
        "Changelog",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    changelog_entry::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(ChangelogEntryAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = changelog_entry::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { changelog_entry::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = changelog_entry::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            changelog_entry::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            changelog_entry::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            changelog_entry::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : roadmap_entry ──
    let meta = AdminResource::new(
        "roadmap_entry",
        "crate::entities::roadmap_entry::Model",
        "AdminForm",
        "Roadmap",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    roadmap_entry::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(RoadmapEntryAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = roadmap_entry::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { roadmap_entry::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = roadmap_entry::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            roadmap_entry::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            roadmap_entry::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            roadmap_entry::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : known_issue ──
    let meta = AdminResource::new(
        "known_issue",
        "crate::entities::known_issue::Model",
        "AdminForm",
        "Problèmes connus",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    known_issue::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(KnownIssueAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = known_issue::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { known_issue::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = known_issue::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            known_issue::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            known_issue::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            known_issue::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : demo_category ──
    let meta = AdminResource::new(
        "demo_category",
        "crate::entities::demo_category::Model",
        "AdminForm",
        "Catégories",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    demo_category::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(DemoCategoryAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = demo_category::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { demo_category::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = demo_category::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_category::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            demo_category::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_category::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : demo_page ──
    let meta = AdminResource::new(
        "demo_page",
        "crate::entities::demo_page::Model",
        "AdminForm",
        "Pages",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = demo_page::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(DemoPageAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = demo_page::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { demo_page::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = demo_page::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_page::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            demo_page::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_page::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : demo_section ──
    let meta = AdminResource::new(
        "demo_section",
        "crate::entities::demo_section::Model",
        "AdminForm",
        "Sections",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    demo_section::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(DemoSectionAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = demo_section::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { demo_section::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = demo_section::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_section::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            demo_section::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_section::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : code_example ──
    let meta = AdminResource::new(
        "code_example",
        "crate::entities::code_example::Model",
        "AdminForm",
        "Exemples de code",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    code_example::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(CodeExampleAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = code_example::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { code_example::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = code_example::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            code_example::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            code_example::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            code_example::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : page_doc_link ──
    let meta = AdminResource::new(
        "page_doc_link",
        "crate::entities::page_doc_link::Model",
        "AdminForm",
        "Liens documentation",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    page_doc_link::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(PageDocLinkAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = page_doc_link::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { page_doc_link::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = page_doc_link::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            page_doc_link::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            page_doc_link::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            page_doc_link::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : form_field ──
    let meta = AdminResource::new(
        "form_field",
        "crate::entities::form_field::Model",
        "AdminForm",
        "Champs formulaire",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = form_field::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(FormFieldAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = form_field::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { form_field::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = form_field::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            form_field::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            form_field::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            form_field::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : doc_section ──
    let meta = AdminResource::new(
        "doc_section",
        "crate::entities::doc_section::Model",
        "AdminForm",
        "Doc — Sections",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    doc_section::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(DocSectionAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = doc_section::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { doc_section::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = doc_section::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_section::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            doc_section::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_section::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : doc_page ──
    let meta = AdminResource::new(
        "doc_page",
        "crate::entities::doc_page::Model",
        "AdminForm",
        "Doc — Pages",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = doc_page::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(DocPageAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = doc_page::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { doc_page::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = doc_page::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_page::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            doc_page::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_page::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().list_filter(vec![("lang", "Langue", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::sea_query::{Alias, Expr, Order, Query};
            use sea_orm::{ConnectionTrait, ExprTrait};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> =
                std::collections::HashMap::new();
            let page_size_lang = 10u64;
            let cur_page_lang = pages.get("lang").copied().unwrap_or(0);
            let count_stmt_lang = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT lang)"))
                .from(Alias::new(doc_page::Entity.table_name()))
                .and_where(Expr::col(Alias::new("lang")).is_not_null())
                .to_owned();
            let count_row_lang = db.query_one(&count_stmt_lang).await.unwrap_or(None);
            let total_lang = count_row_lang
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_lang = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(lang AS TEXT)"))
                .from(Alias::new(doc_page::Entity.table_name()))
                .and_where(Expr::col(Alias::new("lang")).is_not_null())
                .order_by(Alias::new("lang"), Order::Asc)
                .limit(page_size_lang)
                .offset(cur_page_lang * page_size_lang)
                .to_owned();
            let rows_lang = db.query_all(&stmt_lang).await.unwrap_or_default();
            let vals_lang: Vec<String> = rows_lang
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert("lang".to_string(), (vals_lang, total_lang));
            Ok(result)
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn)
            .with_filter_fn(filter_fn),
    );

    // ── Ressource : doc_block ──
    let meta = AdminResource::new(
        "doc_block",
        "crate::entities::doc_block::Model",
        "AdminForm",
        "Doc — Blocs",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = doc_block::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(DocBlockAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = doc_block::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { doc_block::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = doc_block::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_block::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            doc_block::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_block::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().list_filter(vec![
        ("page_id", "page", 10u64),
        ("block_type", "type", 5u64),
        ("heading", "En-tête", 1u64),
    ]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::sea_query::{Alias, Expr, Order, Query};
            use sea_orm::{ConnectionTrait, ExprTrait};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> =
                std::collections::HashMap::new();
            let page_size_page_id = 10u64;
            let cur_page_page_id = pages.get("page_id").copied().unwrap_or(0);
            let count_stmt_page_id = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT page_id)"))
                .from(Alias::new(doc_block::Entity.table_name()))
                .and_where(Expr::col(Alias::new("page_id")).is_not_null())
                .to_owned();
            let count_row_page_id = db.query_one(&count_stmt_page_id).await.unwrap_or(None);
            let total_page_id = count_row_page_id
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_page_id = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(page_id AS TEXT)"))
                .from(Alias::new(doc_block::Entity.table_name()))
                .and_where(Expr::col(Alias::new("page_id")).is_not_null())
                .order_by(Alias::new("page_id"), Order::Asc)
                .limit(page_size_page_id)
                .offset(cur_page_page_id * page_size_page_id)
                .to_owned();
            let rows_page_id = db.query_all(&stmt_page_id).await.unwrap_or_default();
            let vals_page_id: Vec<String> = rows_page_id
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert("page_id".to_string(), (vals_page_id, total_page_id));
            let page_size_block_type = 5u64;
            let cur_page_block_type = pages.get("block_type").copied().unwrap_or(0);
            let count_stmt_block_type = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT block_type)"))
                .from(Alias::new(doc_block::Entity.table_name()))
                .and_where(Expr::col(Alias::new("block_type")).is_not_null())
                .to_owned();
            let count_row_block_type = db.query_one(&count_stmt_block_type).await.unwrap_or(None);
            let total_block_type = count_row_block_type
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_block_type = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(block_type AS TEXT)"))
                .from(Alias::new(doc_block::Entity.table_name()))
                .and_where(Expr::col(Alias::new("block_type")).is_not_null())
                .order_by(Alias::new("block_type"), Order::Asc)
                .limit(page_size_block_type)
                .offset(cur_page_block_type * page_size_block_type)
                .to_owned();
            let rows_block_type = db.query_all(&stmt_block_type).await.unwrap_or_default();
            let vals_block_type: Vec<String> = rows_block_type
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert(
                "block_type".to_string(),
                (vals_block_type, total_block_type),
            );
            let page_size_heading = 1u64;
            let cur_page_heading = pages.get("heading").copied().unwrap_or(0);
            let count_stmt_heading = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT heading)"))
                .from(Alias::new(doc_block::Entity.table_name()))
                .and_where(Expr::col(Alias::new("heading")).is_not_null())
                .to_owned();
            let count_row_heading = db.query_one(&count_stmt_heading).await.unwrap_or(None);
            let total_heading = count_row_heading
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_heading = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(heading AS TEXT)"))
                .from(Alias::new(doc_block::Entity.table_name()))
                .and_where(Expr::col(Alias::new("heading")).is_not_null())
                .order_by(Alias::new("heading"), Order::Asc)
                .limit(page_size_heading)
                .offset(cur_page_heading * page_size_heading)
                .to_owned();
            let rows_heading = db.query_all(&stmt_heading).await.unwrap_or_default();
            let vals_heading: Vec<String> = rows_heading
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert("heading".to_string(), (vals_heading, total_heading));
            Ok(result)
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn)
            .with_filter_fn(filter_fn),
    );

    // ── Ressource : site_config ──
    let meta = AdminResource::new(
        "site_config",
        "crate::entities::site_config::Model",
        "AdminForm",
        "Configuration site",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form =
                    site_config::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(SiteConfigAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = site_config::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { site_config::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = site_config::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            site_config::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            site_config::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            site_config::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn),
    );

    // ── Ressource : cour ──
    let meta = AdminResource::new(
        "cour",
        "crate::entities::cour::Model",
        "AdminForm",
        "Cours",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = cour::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(CourAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = cour::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { cour::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = cour::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            cour::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            cour::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            cour::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    let meta = meta.display(
        DisplayConfig::new()
            .columns_include(vec![
                ("slug", "Slug"),
                ("theme", "Thème"),
                ("difficulte", "Difficulté"),
                ("ordre", "Ordre"),
            ])
            .list_filter(vec![
                ("theme", "Thème", 10u64),
                ("difficulte", "Difficulté", 10u64),
            ]),
    );
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::sea_query::{Alias, Expr, Order, Query};
            use sea_orm::{ConnectionTrait, ExprTrait};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> =
                std::collections::HashMap::new();
            let page_size_theme = 10u64;
            let cur_page_theme = pages.get("theme").copied().unwrap_or(0);
            let count_stmt_theme = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT theme)"))
                .from(Alias::new(cour::Entity.table_name()))
                .and_where(Expr::col(Alias::new("theme")).is_not_null())
                .to_owned();
            let count_row_theme = db.query_one(&count_stmt_theme).await.unwrap_or(None);
            let total_theme = count_row_theme
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_theme = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(theme AS TEXT)"))
                .from(Alias::new(cour::Entity.table_name()))
                .and_where(Expr::col(Alias::new("theme")).is_not_null())
                .order_by(Alias::new("theme"), Order::Asc)
                .limit(page_size_theme)
                .offset(cur_page_theme * page_size_theme)
                .to_owned();
            let rows_theme = db.query_all(&stmt_theme).await.unwrap_or_default();
            let vals_theme: Vec<String> = rows_theme
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert("theme".to_string(), (vals_theme, total_theme));
            let page_size_difficulte = 10u64;
            let cur_page_difficulte = pages.get("difficulte").copied().unwrap_or(0);
            let count_stmt_difficulte = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT difficulte)"))
                .from(Alias::new(cour::Entity.table_name()))
                .and_where(Expr::col(Alias::new("difficulte")).is_not_null())
                .to_owned();
            let count_row_difficulte = db.query_one(&count_stmt_difficulte).await.unwrap_or(None);
            let total_difficulte = count_row_difficulte
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_difficulte = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(difficulte AS TEXT)"))
                .from(Alias::new(cour::Entity.table_name()))
                .and_where(Expr::col(Alias::new("difficulte")).is_not_null())
                .order_by(Alias::new("difficulte"), Order::Asc)
                .limit(page_size_difficulte)
                .offset(cur_page_difficulte * page_size_difficulte)
                .to_owned();
            let rows_difficulte = db.query_all(&stmt_difficulte).await.unwrap_or_default();
            let vals_difficulte: Vec<String> = rows_difficulte
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert(
                "difficulte".to_string(),
                (vals_difficulte, total_difficulte),
            );
            Ok(result)
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn)
            .with_filter_fn(filter_fn),
    );

    // ── Ressource : chapitre ──
    let meta = AdminResource::new(
        "chapitre",
        "crate::entities::chapitre::Model",
        "AdminForm",
        "Chapitres",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = chapitre::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(ChapitreAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = chapitre::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { chapitre::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = chapitre::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            chapitre::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            chapitre::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            chapitre::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().list_filter(vec![("cour_id", "Cours", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::sea_query::{Alias, Expr, Order, Query};
            use sea_orm::{ConnectionTrait, ExprTrait};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> =
                std::collections::HashMap::new();
            let page_size_cour_id = 10u64;
            let cur_page_cour_id = pages.get("cour_id").copied().unwrap_or(0);
            let count_stmt_cour_id = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT cour_id)"))
                .from(Alias::new(chapitre::Entity.table_name()))
                .and_where(Expr::col(Alias::new("cour_id")).is_not_null())
                .to_owned();
            let count_row_cour_id = db.query_one(&count_stmt_cour_id).await.unwrap_or(None);
            let total_cour_id = count_row_cour_id
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_cour_id = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(cour_id AS TEXT)"))
                .from(Alias::new(chapitre::Entity.table_name()))
                .and_where(Expr::col(Alias::new("cour_id")).is_not_null())
                .order_by(Alias::new("cour_id"), Order::Asc)
                .limit(page_size_cour_id)
                .offset(cur_page_cour_id * page_size_cour_id)
                .to_owned();
            let rows_cour_id = db.query_all(&stmt_cour_id).await.unwrap_or_default();
            let vals_cour_id: Vec<String> = rows_cour_id
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert("cour_id".to_string(), (vals_cour_id, total_cour_id));
            Ok(result)
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn)
            .with_filter_fn(filter_fn),
    );

    // ── Ressource : cour_block ──
    let meta = AdminResource::new(
        "cour_block",
        "crate::entities::cour_block::Model",
        "AdminForm",
        "Cours — Blocs",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder =
        Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
            Box::pin(async move {
                let form = cour_block::AdminForm::build_with_data(&data, tera, &csrf, method).await;
                Box::new(CourBlockAdminFormDynWrapper(form)) as Box<dyn DynForm>
            })
        });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{
                QueryFilter,
                sea_query::{Alias, Expr, Order},
            };
            let mut query = cour_block::Entity::find();
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
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
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

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move { cour_block::Entity::find().count(&*db).await })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = cour_block::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            cour_block::Entity::delete_by_id(id)
                .exec(&*db)
                .await
                .map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            cour_block::admin_from_form(&data, None)
                .insert(&*db)
                .await
                .map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id
                .parse::<i32>()
                .map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            cour_block::admin_from_form(&data, Some(id))
                .update(&*db)
                .await
                .map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().list_filter(vec![
        ("chapitre_id", "Chapitre", 10u64),
        ("block_type", "Type", 5u64),
    ]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::sea_query::{Alias, Expr, Order, Query};
            use sea_orm::{ConnectionTrait, ExprTrait};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> =
                std::collections::HashMap::new();
            let page_size_chapitre_id = 10u64;
            let cur_page_chapitre_id = pages.get("chapitre_id").copied().unwrap_or(0);
            let count_stmt_chapitre_id = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT chapitre_id)"))
                .from(Alias::new(cour_block::Entity.table_name()))
                .and_where(Expr::col(Alias::new("chapitre_id")).is_not_null())
                .to_owned();
            let count_row_chapitre_id = db.query_one(&count_stmt_chapitre_id).await.unwrap_or(None);
            let total_chapitre_id = count_row_chapitre_id
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_chapitre_id = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(chapitre_id AS TEXT)"))
                .from(Alias::new(cour_block::Entity.table_name()))
                .and_where(Expr::col(Alias::new("chapitre_id")).is_not_null())
                .order_by(Alias::new("chapitre_id"), Order::Asc)
                .limit(page_size_chapitre_id)
                .offset(cur_page_chapitre_id * page_size_chapitre_id)
                .to_owned();
            let rows_chapitre_id = db.query_all(&stmt_chapitre_id).await.unwrap_or_default();
            let vals_chapitre_id: Vec<String> = rows_chapitre_id
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert(
                "chapitre_id".to_string(),
                (vals_chapitre_id, total_chapitre_id),
            );
            let page_size_block_type = 5u64;
            let cur_page_block_type = pages.get("block_type").copied().unwrap_or(0);
            let count_stmt_block_type = Query::select()
                .expr(Expr::cust("COUNT(DISTINCT block_type)"))
                .from(Alias::new(cour_block::Entity.table_name()))
                .and_where(Expr::col(Alias::new("block_type")).is_not_null())
                .to_owned();
            let count_row_block_type = db.query_one(&count_stmt_block_type).await.unwrap_or(None);
            let total_block_type = count_row_block_type
                .and_then(|r| r.try_get_by_index::<i64>(0).ok())
                .unwrap_or(0) as u64;
            let stmt_block_type = Query::select()
                .distinct()
                .expr(Expr::cust("CAST(block_type AS TEXT)"))
                .from(Alias::new(cour_block::Entity.table_name()))
                .and_where(Expr::col(Alias::new("block_type")).is_not_null())
                .order_by(Alias::new("block_type"), Order::Asc)
                .limit(page_size_block_type)
                .offset(cur_page_block_type * page_size_block_type)
                .to_owned();
            let rows_block_type = db.query_all(&stmt_block_type).await.unwrap_or_default();
            let vals_block_type: Vec<String> = rows_block_type
                .iter()
                .filter_map(|r| r.try_get_by_index::<String>(0).ok())
                .collect();
            result.insert(
                "block_type".to_string(),
                (vals_block_type, total_block_type),
            );
            Ok(result)
        })
    });

    registry.register(
        ResourceEntry::new(meta, form_builder)
            .with_list_fn(list_fn)
            .with_get_fn(get_fn)
            .with_delete_fn(delete_fn)
            .with_create_fn(create_fn)
            .with_update_fn(update_fn)
            .with_count_fn(count_fn)
            .with_filter_fn(filter_fn),
    );

    registry
}

/// Construit le Router axum du prototype admin pour le préfixe donné.
/// À passer à `.with_admin(|a| a.routes(admins::routes("/admin")))` dans main.rs.
pub fn routes(prefix: &str) -> runique::axum::Router {
    let p = prefix.trim_end_matches('/');
    runique::axum::Router::new()
        .route(
            &format!("{}/{{resource}}/{{action}}", p),
            get(admin_get).post(admin_post),
        )
        .route(
            &format!("{}/{{resource}}/{{id}}/{{action}}", p),
            get(admin_get_id).post(admin_post_id),
        )
}

/// Retourne l'état partagé du prototype admin (pour le dashboard).
pub fn admin_state() -> std::sync::Arc<PrototypeAdminState> {
    let config = Arc::new(AdminConfig::new());
    std::sync::Arc::new(PrototypeAdminState {
        registry: Arc::new(admin_register()),
        config,
    })
}
