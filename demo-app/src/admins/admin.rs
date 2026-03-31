// AUTO-admin — DO NOT EDIT MANUALLY
// admin by `runique start` from src/admin.rs

use runique::prelude::*;
use runique::admin::resource_entry::FilterFn;

use crate::entities::users;
use crate::entities::contribution;
use crate::entities::blog;
use crate::entities::changelog_entry;
use crate::entities::roadmap_entry;
use crate::entities::known_issue;
use crate::entities::demo_category;
use crate::entities::demo_page;
use crate::entities::demo_section;
use crate::entities::code_example;
use crate::entities::page_doc_link;
use crate::entities::form_field;
use crate::entities::doc_section;
use crate::entities::doc_page;
use crate::entities::doc_block;
use crate::entities::site_config;
use crate::entities::cour;
use crate::entities::chapitre;
use crate::entities::cour_block;
use crate::entities::runique_release;

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

// ── DynForm wrapper pour contribution::AdminForm ──
struct ContributionAdminFormDynWrapper(pub contribution::AdminForm);

#[async_trait]
impl DynForm for ContributionAdminFormDynWrapper {
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

// ── DynForm wrapper pour runique_release::AdminForm ──
struct RuniqueReleaseAdminFormDynWrapper(pub runique_release::AdminForm);

#[async_trait]
impl DynForm for RuniqueReleaseAdminFormDynWrapper {
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
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = users::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(UsersAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = users::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            users::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = users::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            users::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            users::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            users::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let edit_form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = crate::formulaire::UserEditForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(UsersEditFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("username", "Nom d'utilisateur"), ("email", "Email"), ("is_superuser", "Superuser"), ("is_active", "Actif")]).list_filter(vec![("username", "Nom d'utilisateur", 10u64), ("email", "Email", 10u64), ("is_superuser", "Superuser", 10u64), ("is_active", "Actif", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_username = 10u64;
            let cur_page_username = pages.get("username").copied().unwrap_or(0);
            let count_stmt_username = Query::select().expr(Expr::cust("COUNT(DISTINCT username)")).from(Alias::new(users::Entity.table_name())).and_where(Expr::col(Alias::new("username")).is_not_null()).to_owned();
            let count_row_username = match db.query_one(&count_stmt_username).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `users.username` : colonne introuvable en DB — {}", e); None } };
            let total_username = count_row_username.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_username = Query::select().distinct().expr(Expr::cust("CAST(username AS TEXT)")).from(Alias::new(users::Entity.table_name())).and_where(Expr::col(Alias::new("username")).is_not_null()).limit(page_size_username).offset(cur_page_username * page_size_username).to_owned();
            let rows_username = match db.query_all(&stmt_username).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `users.username` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_username: Vec<String> = rows_username.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_username.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("username".to_string(), (vals_username, total_username));
            let page_size_email = 10u64;
            let cur_page_email = pages.get("email").copied().unwrap_or(0);
            let count_stmt_email = Query::select().expr(Expr::cust("COUNT(DISTINCT email)")).from(Alias::new(users::Entity.table_name())).and_where(Expr::col(Alias::new("email")).is_not_null()).to_owned();
            let count_row_email = match db.query_one(&count_stmt_email).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `users.email` : colonne introuvable en DB — {}", e); None } };
            let total_email = count_row_email.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_email = Query::select().distinct().expr(Expr::cust("CAST(email AS TEXT)")).from(Alias::new(users::Entity.table_name())).and_where(Expr::col(Alias::new("email")).is_not_null()).limit(page_size_email).offset(cur_page_email * page_size_email).to_owned();
            let rows_email = match db.query_all(&stmt_email).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `users.email` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_email: Vec<String> = rows_email.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_email.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("email".to_string(), (vals_email, total_email));
            let page_size_is_superuser = 10u64;
            let cur_page_is_superuser = pages.get("is_superuser").copied().unwrap_or(0);
            let count_stmt_is_superuser = Query::select().expr(Expr::cust("COUNT(DISTINCT is_superuser)")).from(Alias::new(users::Entity.table_name())).and_where(Expr::col(Alias::new("is_superuser")).is_not_null()).to_owned();
            let count_row_is_superuser = match db.query_one(&count_stmt_is_superuser).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `users.is_superuser` : colonne introuvable en DB — {}", e); None } };
            let total_is_superuser = count_row_is_superuser.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_is_superuser = Query::select().distinct().expr(Expr::cust("CAST(is_superuser AS TEXT)")).from(Alias::new(users::Entity.table_name())).and_where(Expr::col(Alias::new("is_superuser")).is_not_null()).limit(page_size_is_superuser).offset(cur_page_is_superuser * page_size_is_superuser).to_owned();
            let rows_is_superuser = match db.query_all(&stmt_is_superuser).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `users.is_superuser` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_is_superuser: Vec<String> = rows_is_superuser.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_is_superuser.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("is_superuser".to_string(), (vals_is_superuser, total_is_superuser));
            let page_size_is_active = 10u64;
            let cur_page_is_active = pages.get("is_active").copied().unwrap_or(0);
            let count_stmt_is_active = Query::select().expr(Expr::cust("COUNT(DISTINCT is_active)")).from(Alias::new(users::Entity.table_name())).and_where(Expr::col(Alias::new("is_active")).is_not_null()).to_owned();
            let count_row_is_active = match db.query_one(&count_stmt_is_active).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `users.is_active` : colonne introuvable en DB — {}", e); None } };
            let total_is_active = count_row_is_active.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_is_active = Query::select().distinct().expr(Expr::cust("CAST(is_active AS TEXT)")).from(Alias::new(users::Entity.table_name())).and_where(Expr::col(Alias::new("is_active")).is_not_null()).limit(page_size_is_active).offset(cur_page_is_active * page_size_is_active).to_owned();
            let rows_is_active = match db.query_all(&stmt_is_active).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `users.is_active` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_is_active: Vec<String> = rows_is_active.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_is_active.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : contribution ──
    let meta = AdminResource::new(
        "contribution",
        "crate::entities::contribution::Model",
        "AdminForm",
        "Contribution",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = contribution::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(ContributionAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = contribution::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            contribution::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = contribution::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            contribution::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            contribution::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            contribution::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("user_id", "contributeur"), ("contribution_type", "type"), ("title", "titre"), ("content", "contenu")]).list_filter(vec![("user_id", "contributeur", 5u64), ("contribution_type", "type", 5u64), ("title", "titre", 5u64), ("content", "contenu", 5u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_user_id = 5u64;
            let cur_page_user_id = pages.get("user_id").copied().unwrap_or(0);
            let count_stmt_user_id = Query::select().expr(Expr::cust("COUNT(DISTINCT user_id)")).from(Alias::new(contribution::Entity.table_name())).and_where(Expr::col(Alias::new("user_id")).is_not_null()).to_owned();
            let count_row_user_id = match db.query_one(&count_stmt_user_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `contribution.user_id` : colonne introuvable en DB — {}", e); None } };
            let total_user_id = count_row_user_id.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_user_id = Query::select().distinct().expr(Expr::cust("CAST(user_id AS TEXT)")).from(Alias::new(contribution::Entity.table_name())).and_where(Expr::col(Alias::new("user_id")).is_not_null()).limit(page_size_user_id).offset(cur_page_user_id * page_size_user_id).to_owned();
            let rows_user_id = match db.query_all(&stmt_user_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `contribution.user_id` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_user_id: Vec<String> = rows_user_id.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_user_id.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("user_id".to_string(), (vals_user_id, total_user_id));
            let page_size_contribution_type = 5u64;
            let cur_page_contribution_type = pages.get("contribution_type").copied().unwrap_or(0);
            let count_stmt_contribution_type = Query::select().expr(Expr::cust("COUNT(DISTINCT contribution_type)")).from(Alias::new(contribution::Entity.table_name())).and_where(Expr::col(Alias::new("contribution_type")).is_not_null()).to_owned();
            let count_row_contribution_type = match db.query_one(&count_stmt_contribution_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `contribution.contribution_type` : colonne introuvable en DB — {}", e); None } };
            let total_contribution_type = count_row_contribution_type.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_contribution_type = Query::select().distinct().expr(Expr::cust("CAST(contribution_type AS TEXT)")).from(Alias::new(contribution::Entity.table_name())).and_where(Expr::col(Alias::new("contribution_type")).is_not_null()).limit(page_size_contribution_type).offset(cur_page_contribution_type * page_size_contribution_type).to_owned();
            let rows_contribution_type = match db.query_all(&stmt_contribution_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `contribution.contribution_type` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_contribution_type: Vec<String> = rows_contribution_type.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_contribution_type.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("contribution_type".to_string(), (vals_contribution_type, total_contribution_type));
            let page_size_title = 5u64;
            let cur_page_title = pages.get("title").copied().unwrap_or(0);
            let count_stmt_title = Query::select().expr(Expr::cust("COUNT(DISTINCT title)")).from(Alias::new(contribution::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).to_owned();
            let count_row_title = match db.query_one(&count_stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `contribution.title` : colonne introuvable en DB — {}", e); None } };
            let total_title = count_row_title.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_title = Query::select().distinct().expr(Expr::cust("CAST(title AS TEXT)")).from(Alias::new(contribution::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).limit(page_size_title).offset(cur_page_title * page_size_title).to_owned();
            let rows_title = match db.query_all(&stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `contribution.title` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_title: Vec<String> = rows_title.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_title.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("title".to_string(), (vals_title, total_title));
            let page_size_content = 5u64;
            let cur_page_content = pages.get("content").copied().unwrap_or(0);
            let count_stmt_content = Query::select().expr(Expr::cust("COUNT(DISTINCT content)")).from(Alias::new(contribution::Entity.table_name())).and_where(Expr::col(Alias::new("content")).is_not_null()).to_owned();
            let count_row_content = match db.query_one(&count_stmt_content).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `contribution.content` : colonne introuvable en DB — {}", e); None } };
            let total_content = count_row_content.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_content = Query::select().distinct().expr(Expr::cust("CAST(content AS TEXT)")).from(Alias::new(contribution::Entity.table_name())).and_where(Expr::col(Alias::new("content")).is_not_null()).limit(page_size_content).offset(cur_page_content * page_size_content).to_owned();
            let rows_content = match db.query_all(&stmt_content).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `contribution.content` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_content: Vec<String> = rows_content.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_content.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("content".to_string(), (vals_content, total_content));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : blog ──
    let meta = AdminResource::new(
        "blog",
        "crate::entities::blog::Model",
        "AdminForm",
        "Articles",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = blog::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(BlogAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = blog::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            blog::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = blog::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            blog::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            blog::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            blog::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("title", "Titre"), ("email", "email"), ("website", "lien url"), ("summary", "Sujet"), ("content", "contenu")]).list_filter(vec![("title", "Titre", 10u64), ("email", "email", 10u64), ("website", "lien url", 10u64), ("summary", "Sujet", 10u64), ("content", "contenu", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_title = 10u64;
            let cur_page_title = pages.get("title").copied().unwrap_or(0);
            let count_stmt_title = Query::select().expr(Expr::cust("COUNT(DISTINCT title)")).from(Alias::new(blog::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).to_owned();
            let count_row_title = match db.query_one(&count_stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `blog.title` : colonne introuvable en DB — {}", e); None } };
            let total_title = count_row_title.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_title = Query::select().distinct().expr(Expr::cust("CAST(title AS TEXT)")).from(Alias::new(blog::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).limit(page_size_title).offset(cur_page_title * page_size_title).to_owned();
            let rows_title = match db.query_all(&stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `blog.title` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_title: Vec<String> = rows_title.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_title.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("title".to_string(), (vals_title, total_title));
            let page_size_email = 10u64;
            let cur_page_email = pages.get("email").copied().unwrap_or(0);
            let count_stmt_email = Query::select().expr(Expr::cust("COUNT(DISTINCT email)")).from(Alias::new(blog::Entity.table_name())).and_where(Expr::col(Alias::new("email")).is_not_null()).to_owned();
            let count_row_email = match db.query_one(&count_stmt_email).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `blog.email` : colonne introuvable en DB — {}", e); None } };
            let total_email = count_row_email.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_email = Query::select().distinct().expr(Expr::cust("CAST(email AS TEXT)")).from(Alias::new(blog::Entity.table_name())).and_where(Expr::col(Alias::new("email")).is_not_null()).limit(page_size_email).offset(cur_page_email * page_size_email).to_owned();
            let rows_email = match db.query_all(&stmt_email).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `blog.email` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_email: Vec<String> = rows_email.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_email.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("email".to_string(), (vals_email, total_email));
            let page_size_website = 10u64;
            let cur_page_website = pages.get("website").copied().unwrap_or(0);
            let count_stmt_website = Query::select().expr(Expr::cust("COUNT(DISTINCT website)")).from(Alias::new(blog::Entity.table_name())).and_where(Expr::col(Alias::new("website")).is_not_null()).to_owned();
            let count_row_website = match db.query_one(&count_stmt_website).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `blog.website` : colonne introuvable en DB — {}", e); None } };
            let total_website = count_row_website.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_website = Query::select().distinct().expr(Expr::cust("CAST(website AS TEXT)")).from(Alias::new(blog::Entity.table_name())).and_where(Expr::col(Alias::new("website")).is_not_null()).limit(page_size_website).offset(cur_page_website * page_size_website).to_owned();
            let rows_website = match db.query_all(&stmt_website).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `blog.website` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_website: Vec<String> = rows_website.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_website.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("website".to_string(), (vals_website, total_website));
            let page_size_summary = 10u64;
            let cur_page_summary = pages.get("summary").copied().unwrap_or(0);
            let count_stmt_summary = Query::select().expr(Expr::cust("COUNT(DISTINCT summary)")).from(Alias::new(blog::Entity.table_name())).and_where(Expr::col(Alias::new("summary")).is_not_null()).to_owned();
            let count_row_summary = match db.query_one(&count_stmt_summary).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `blog.summary` : colonne introuvable en DB — {}", e); None } };
            let total_summary = count_row_summary.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_summary = Query::select().distinct().expr(Expr::cust("CAST(summary AS TEXT)")).from(Alias::new(blog::Entity.table_name())).and_where(Expr::col(Alias::new("summary")).is_not_null()).limit(page_size_summary).offset(cur_page_summary * page_size_summary).to_owned();
            let rows_summary = match db.query_all(&stmt_summary).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `blog.summary` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_summary: Vec<String> = rows_summary.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_summary.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("summary".to_string(), (vals_summary, total_summary));
            let page_size_content = 10u64;
            let cur_page_content = pages.get("content").copied().unwrap_or(0);
            let count_stmt_content = Query::select().expr(Expr::cust("COUNT(DISTINCT content)")).from(Alias::new(blog::Entity.table_name())).and_where(Expr::col(Alias::new("content")).is_not_null()).to_owned();
            let count_row_content = match db.query_one(&count_stmt_content).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `blog.content` : colonne introuvable en DB — {}", e); None } };
            let total_content = count_row_content.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_content = Query::select().distinct().expr(Expr::cust("CAST(content AS TEXT)")).from(Alias::new(blog::Entity.table_name())).and_where(Expr::col(Alias::new("content")).is_not_null()).limit(page_size_content).offset(cur_page_content * page_size_content).to_owned();
            let rows_content = match db.query_all(&stmt_content).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `blog.content` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_content: Vec<String> = rows_content.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_content.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("content".to_string(), (vals_content, total_content));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : changelog_entry ──
    let meta = AdminResource::new(
        "changelog_entry",
        "crate::entities::changelog_entry::Model",
        "AdminForm",
        "Changelog",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = changelog_entry::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(ChangelogEntryAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = changelog_entry::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            changelog_entry::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = changelog_entry::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            changelog_entry::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            changelog_entry::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            changelog_entry::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("version", "Version"), ("release_date", "Date de sortie"), ("category", "Catégorie"), ("title", "Titre"), ("description", "Description"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("version", "Version", 10u64), ("release_date", "Date", 10u64), ("category", "Catégorie", 10u64), ("title", "Titre", 10u64), ("description", "Description", 10u64), ("sort_order", "Ordre d'affichage", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_version = 10u64;
            let cur_page_version = pages.get("version").copied().unwrap_or(0);
            let count_stmt_version = Query::select().expr(Expr::cust("COUNT(DISTINCT version)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("version")).is_not_null()).to_owned();
            let count_row_version = match db.query_one(&count_stmt_version).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.version` : colonne introuvable en DB — {}", e); None } };
            let total_version = count_row_version.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_version = Query::select().distinct().expr(Expr::cust("CAST(version AS TEXT)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("version")).is_not_null()).limit(page_size_version).offset(cur_page_version * page_size_version).to_owned();
            let rows_version = match db.query_all(&stmt_version).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.version` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_version: Vec<String> = rows_version.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_version.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("version".to_string(), (vals_version, total_version));
            let page_size_release_date = 10u64;
            let cur_page_release_date = pages.get("release_date").copied().unwrap_or(0);
            let count_stmt_release_date = Query::select().expr(Expr::cust("COUNT(DISTINCT release_date)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("release_date")).is_not_null()).to_owned();
            let count_row_release_date = match db.query_one(&count_stmt_release_date).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.release_date` : colonne introuvable en DB — {}", e); None } };
            let total_release_date = count_row_release_date.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_release_date = Query::select().distinct().expr(Expr::cust("CAST(release_date AS TEXT)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("release_date")).is_not_null()).limit(page_size_release_date).offset(cur_page_release_date * page_size_release_date).to_owned();
            let rows_release_date = match db.query_all(&stmt_release_date).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.release_date` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_release_date: Vec<String> = rows_release_date.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_release_date.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("release_date".to_string(), (vals_release_date, total_release_date));
            let page_size_category = 10u64;
            let cur_page_category = pages.get("category").copied().unwrap_or(0);
            let count_stmt_category = Query::select().expr(Expr::cust("COUNT(DISTINCT category)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("category")).is_not_null()).to_owned();
            let count_row_category = match db.query_one(&count_stmt_category).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.category` : colonne introuvable en DB — {}", e); None } };
            let total_category = count_row_category.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_category = Query::select().distinct().expr(Expr::cust("CAST(category AS TEXT)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("category")).is_not_null()).limit(page_size_category).offset(cur_page_category * page_size_category).to_owned();
            let rows_category = match db.query_all(&stmt_category).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.category` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_category: Vec<String> = rows_category.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_category.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("category".to_string(), (vals_category, total_category));
            let page_size_title = 10u64;
            let cur_page_title = pages.get("title").copied().unwrap_or(0);
            let count_stmt_title = Query::select().expr(Expr::cust("COUNT(DISTINCT title)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).to_owned();
            let count_row_title = match db.query_one(&count_stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.title` : colonne introuvable en DB — {}", e); None } };
            let total_title = count_row_title.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_title = Query::select().distinct().expr(Expr::cust("CAST(title AS TEXT)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).limit(page_size_title).offset(cur_page_title * page_size_title).to_owned();
            let rows_title = match db.query_all(&stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.title` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_title: Vec<String> = rows_title.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_title.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("title".to_string(), (vals_title, total_title));
            let page_size_description = 10u64;
            let cur_page_description = pages.get("description").copied().unwrap_or(0);
            let count_stmt_description = Query::select().expr(Expr::cust("COUNT(DISTINCT description)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("description")).is_not_null()).to_owned();
            let count_row_description = match db.query_one(&count_stmt_description).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.description` : colonne introuvable en DB — {}", e); None } };
            let total_description = count_row_description.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_description = Query::select().distinct().expr(Expr::cust("CAST(description AS TEXT)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("description")).is_not_null()).limit(page_size_description).offset(cur_page_description * page_size_description).to_owned();
            let rows_description = match db.query_all(&stmt_description).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.description` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_description: Vec<String> = rows_description.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_description.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("description".to_string(), (vals_description, total_description));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(changelog_entry::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `changelog_entry.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : roadmap_entry ──
    let meta = AdminResource::new(
        "roadmap_entry",
        "crate::entities::roadmap_entry::Model",
        "AdminForm",
        "Roadmap",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = roadmap_entry::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(RoadmapEntryAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = roadmap_entry::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            roadmap_entry::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = roadmap_entry::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            roadmap_entry::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            roadmap_entry::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            roadmap_entry::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("status", "Statut"), ("title", "Titre"), ("description", "Description"), ("link_url", "URL"), ("link_label", "Label"), ("link_url_2", "URL 2"), ("link_label_2", "Label 2"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("status", "Statut", 10u64), ("title", "Titre", 10u64), ("description", "Description", 10u64), ("link_url", "URL", 10u64), ("link_label", "Label", 10u64), ("link_url_2", "URL 2", 10u64), ("link_label_2", "Label 2", 10u64), ("sort_order", "Ordre d'affichage", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_status = 10u64;
            let cur_page_status = pages.get("status").copied().unwrap_or(0);
            let count_stmt_status = Query::select().expr(Expr::cust("COUNT(DISTINCT status)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("status")).is_not_null()).to_owned();
            let count_row_status = match db.query_one(&count_stmt_status).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.status` : colonne introuvable en DB — {}", e); None } };
            let total_status = count_row_status.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_status = Query::select().distinct().expr(Expr::cust("CAST(status AS TEXT)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("status")).is_not_null()).limit(page_size_status).offset(cur_page_status * page_size_status).to_owned();
            let rows_status = match db.query_all(&stmt_status).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.status` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_status: Vec<String> = rows_status.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_status.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("status".to_string(), (vals_status, total_status));
            let page_size_title = 10u64;
            let cur_page_title = pages.get("title").copied().unwrap_or(0);
            let count_stmt_title = Query::select().expr(Expr::cust("COUNT(DISTINCT title)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).to_owned();
            let count_row_title = match db.query_one(&count_stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.title` : colonne introuvable en DB — {}", e); None } };
            let total_title = count_row_title.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_title = Query::select().distinct().expr(Expr::cust("CAST(title AS TEXT)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).limit(page_size_title).offset(cur_page_title * page_size_title).to_owned();
            let rows_title = match db.query_all(&stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.title` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_title: Vec<String> = rows_title.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_title.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("title".to_string(), (vals_title, total_title));
            let page_size_description = 10u64;
            let cur_page_description = pages.get("description").copied().unwrap_or(0);
            let count_stmt_description = Query::select().expr(Expr::cust("COUNT(DISTINCT description)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("description")).is_not_null()).to_owned();
            let count_row_description = match db.query_one(&count_stmt_description).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.description` : colonne introuvable en DB — {}", e); None } };
            let total_description = count_row_description.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_description = Query::select().distinct().expr(Expr::cust("CAST(description AS TEXT)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("description")).is_not_null()).limit(page_size_description).offset(cur_page_description * page_size_description).to_owned();
            let rows_description = match db.query_all(&stmt_description).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.description` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_description: Vec<String> = rows_description.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_description.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("description".to_string(), (vals_description, total_description));
            let page_size_link_url = 10u64;
            let cur_page_link_url = pages.get("link_url").copied().unwrap_or(0);
            let count_stmt_link_url = Query::select().expr(Expr::cust("COUNT(DISTINCT link_url)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("link_url")).is_not_null()).to_owned();
            let count_row_link_url = match db.query_one(&count_stmt_link_url).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.link_url` : colonne introuvable en DB — {}", e); None } };
            let total_link_url = count_row_link_url.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_link_url = Query::select().distinct().expr(Expr::cust("CAST(link_url AS TEXT)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("link_url")).is_not_null()).limit(page_size_link_url).offset(cur_page_link_url * page_size_link_url).to_owned();
            let rows_link_url = match db.query_all(&stmt_link_url).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.link_url` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_link_url: Vec<String> = rows_link_url.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_link_url.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("link_url".to_string(), (vals_link_url, total_link_url));
            let page_size_link_label = 10u64;
            let cur_page_link_label = pages.get("link_label").copied().unwrap_or(0);
            let count_stmt_link_label = Query::select().expr(Expr::cust("COUNT(DISTINCT link_label)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("link_label")).is_not_null()).to_owned();
            let count_row_link_label = match db.query_one(&count_stmt_link_label).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.link_label` : colonne introuvable en DB — {}", e); None } };
            let total_link_label = count_row_link_label.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_link_label = Query::select().distinct().expr(Expr::cust("CAST(link_label AS TEXT)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("link_label")).is_not_null()).limit(page_size_link_label).offset(cur_page_link_label * page_size_link_label).to_owned();
            let rows_link_label = match db.query_all(&stmt_link_label).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.link_label` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_link_label: Vec<String> = rows_link_label.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_link_label.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("link_label".to_string(), (vals_link_label, total_link_label));
            let page_size_link_url_2 = 10u64;
            let cur_page_link_url_2 = pages.get("link_url_2").copied().unwrap_or(0);
            let count_stmt_link_url_2 = Query::select().expr(Expr::cust("COUNT(DISTINCT link_url_2)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("link_url_2")).is_not_null()).to_owned();
            let count_row_link_url_2 = match db.query_one(&count_stmt_link_url_2).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.link_url_2` : colonne introuvable en DB — {}", e); None } };
            let total_link_url_2 = count_row_link_url_2.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_link_url_2 = Query::select().distinct().expr(Expr::cust("CAST(link_url_2 AS TEXT)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("link_url_2")).is_not_null()).limit(page_size_link_url_2).offset(cur_page_link_url_2 * page_size_link_url_2).to_owned();
            let rows_link_url_2 = match db.query_all(&stmt_link_url_2).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.link_url_2` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_link_url_2: Vec<String> = rows_link_url_2.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_link_url_2.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("link_url_2".to_string(), (vals_link_url_2, total_link_url_2));
            let page_size_link_label_2 = 10u64;
            let cur_page_link_label_2 = pages.get("link_label_2").copied().unwrap_or(0);
            let count_stmt_link_label_2 = Query::select().expr(Expr::cust("COUNT(DISTINCT link_label_2)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("link_label_2")).is_not_null()).to_owned();
            let count_row_link_label_2 = match db.query_one(&count_stmt_link_label_2).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.link_label_2` : colonne introuvable en DB — {}", e); None } };
            let total_link_label_2 = count_row_link_label_2.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_link_label_2 = Query::select().distinct().expr(Expr::cust("CAST(link_label_2 AS TEXT)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("link_label_2")).is_not_null()).limit(page_size_link_label_2).offset(cur_page_link_label_2 * page_size_link_label_2).to_owned();
            let rows_link_label_2 = match db.query_all(&stmt_link_label_2).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.link_label_2` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_link_label_2: Vec<String> = rows_link_label_2.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_link_label_2.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("link_label_2".to_string(), (vals_link_label_2, total_link_label_2));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(roadmap_entry::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `roadmap_entry.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : known_issue ──
    let meta = AdminResource::new(
        "known_issue",
        "crate::entities::known_issue::Model",
        "AdminForm",
        "Problèmes connus",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = known_issue::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(KnownIssueAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = known_issue::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            known_issue::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = known_issue::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            known_issue::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            known_issue::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            known_issue::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("version", "Version"), ("title", "Titre"), ("description", "Description"), ("issue_type", "Type"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("version", "Version", 10u64), ("title", "Titre", 10u64), ("description", "Description", 10u64), ("issue_type", "Type", 10u64), ("sort_order", "Ordre d'affichage", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_version = 10u64;
            let cur_page_version = pages.get("version").copied().unwrap_or(0);
            let count_stmt_version = Query::select().expr(Expr::cust("COUNT(DISTINCT version)")).from(Alias::new(known_issue::Entity.table_name())).and_where(Expr::col(Alias::new("version")).is_not_null()).to_owned();
            let count_row_version = match db.query_one(&count_stmt_version).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `known_issue.version` : colonne introuvable en DB — {}", e); None } };
            let total_version = count_row_version.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_version = Query::select().distinct().expr(Expr::cust("CAST(version AS TEXT)")).from(Alias::new(known_issue::Entity.table_name())).and_where(Expr::col(Alias::new("version")).is_not_null()).limit(page_size_version).offset(cur_page_version * page_size_version).to_owned();
            let rows_version = match db.query_all(&stmt_version).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `known_issue.version` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_version: Vec<String> = rows_version.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_version.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("version".to_string(), (vals_version, total_version));
            let page_size_title = 10u64;
            let cur_page_title = pages.get("title").copied().unwrap_or(0);
            let count_stmt_title = Query::select().expr(Expr::cust("COUNT(DISTINCT title)")).from(Alias::new(known_issue::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).to_owned();
            let count_row_title = match db.query_one(&count_stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `known_issue.title` : colonne introuvable en DB — {}", e); None } };
            let total_title = count_row_title.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_title = Query::select().distinct().expr(Expr::cust("CAST(title AS TEXT)")).from(Alias::new(known_issue::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).limit(page_size_title).offset(cur_page_title * page_size_title).to_owned();
            let rows_title = match db.query_all(&stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `known_issue.title` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_title: Vec<String> = rows_title.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_title.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("title".to_string(), (vals_title, total_title));
            let page_size_description = 10u64;
            let cur_page_description = pages.get("description").copied().unwrap_or(0);
            let count_stmt_description = Query::select().expr(Expr::cust("COUNT(DISTINCT description)")).from(Alias::new(known_issue::Entity.table_name())).and_where(Expr::col(Alias::new("description")).is_not_null()).to_owned();
            let count_row_description = match db.query_one(&count_stmt_description).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `known_issue.description` : colonne introuvable en DB — {}", e); None } };
            let total_description = count_row_description.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_description = Query::select().distinct().expr(Expr::cust("CAST(description AS TEXT)")).from(Alias::new(known_issue::Entity.table_name())).and_where(Expr::col(Alias::new("description")).is_not_null()).limit(page_size_description).offset(cur_page_description * page_size_description).to_owned();
            let rows_description = match db.query_all(&stmt_description).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `known_issue.description` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_description: Vec<String> = rows_description.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_description.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("description".to_string(), (vals_description, total_description));
            let page_size_issue_type = 10u64;
            let cur_page_issue_type = pages.get("issue_type").copied().unwrap_or(0);
            let count_stmt_issue_type = Query::select().expr(Expr::cust("COUNT(DISTINCT issue_type)")).from(Alias::new(known_issue::Entity.table_name())).and_where(Expr::col(Alias::new("issue_type")).is_not_null()).to_owned();
            let count_row_issue_type = match db.query_one(&count_stmt_issue_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `known_issue.issue_type` : colonne introuvable en DB — {}", e); None } };
            let total_issue_type = count_row_issue_type.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_issue_type = Query::select().distinct().expr(Expr::cust("CAST(issue_type AS TEXT)")).from(Alias::new(known_issue::Entity.table_name())).and_where(Expr::col(Alias::new("issue_type")).is_not_null()).limit(page_size_issue_type).offset(cur_page_issue_type * page_size_issue_type).to_owned();
            let rows_issue_type = match db.query_all(&stmt_issue_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `known_issue.issue_type` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_issue_type: Vec<String> = rows_issue_type.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_issue_type.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("issue_type".to_string(), (vals_issue_type, total_issue_type));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(known_issue::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `known_issue.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(known_issue::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `known_issue.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : demo_category ──
    let meta = AdminResource::new(
        "demo_category",
        "crate::entities::demo_category::Model",
        "AdminForm",
        "Catégories",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = demo_category::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(DemoCategoryAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = demo_category::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            demo_category::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = demo_category::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_category::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            demo_category::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_category::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
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
    );

    // ── Ressource : demo_page ──
    let meta = AdminResource::new(
        "demo_page",
        "crate::entities::demo_page::Model",
        "AdminForm",
        "Pages",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = demo_page::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(DemoPageAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = demo_page::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            demo_page::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = demo_page::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_page::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            demo_page::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_page::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("category_id", "Catégorie"), ("slug", "Slug"), ("title", "Titre"), ("lead", "Lead"), ("page_type", "Type"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("category_id", "Catégorie", 10u64), ("slug", "Slug", 10u64), ("title", "Titre", 10u64), ("lead", "Lead", 10u64), ("page_type", "Type", 10u64), ("sort_order", "Ordre d'affichage", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_category_id = 10u64;
            let cur_page_category_id = pages.get("category_id").copied().unwrap_or(0);
            let count_stmt_category_id = Query::select().expr(Expr::cust("COUNT(DISTINCT category_id)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("category_id")).is_not_null()).to_owned();
            let count_row_category_id = match db.query_one(&count_stmt_category_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.category_id` : colonne introuvable en DB — {}", e); None } };
            let total_category_id = count_row_category_id.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_category_id = Query::select().distinct().expr(Expr::cust("CAST(category_id AS TEXT)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("category_id")).is_not_null()).limit(page_size_category_id).offset(cur_page_category_id * page_size_category_id).to_owned();
            let rows_category_id = match db.query_all(&stmt_category_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.category_id` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_category_id: Vec<String> = rows_category_id.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_category_id.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("category_id".to_string(), (vals_category_id, total_category_id));
            let page_size_slug = 10u64;
            let cur_page_slug = pages.get("slug").copied().unwrap_or(0);
            let count_stmt_slug = Query::select().expr(Expr::cust("COUNT(DISTINCT slug)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("slug")).is_not_null()).to_owned();
            let count_row_slug = match db.query_one(&count_stmt_slug).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.slug` : colonne introuvable en DB — {}", e); None } };
            let total_slug = count_row_slug.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_slug = Query::select().distinct().expr(Expr::cust("CAST(slug AS TEXT)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("slug")).is_not_null()).limit(page_size_slug).offset(cur_page_slug * page_size_slug).to_owned();
            let rows_slug = match db.query_all(&stmt_slug).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.slug` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_slug: Vec<String> = rows_slug.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_slug.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("slug".to_string(), (vals_slug, total_slug));
            let page_size_title = 10u64;
            let cur_page_title = pages.get("title").copied().unwrap_or(0);
            let count_stmt_title = Query::select().expr(Expr::cust("COUNT(DISTINCT title)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).to_owned();
            let count_row_title = match db.query_one(&count_stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.title` : colonne introuvable en DB — {}", e); None } };
            let total_title = count_row_title.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_title = Query::select().distinct().expr(Expr::cust("CAST(title AS TEXT)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).limit(page_size_title).offset(cur_page_title * page_size_title).to_owned();
            let rows_title = match db.query_all(&stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.title` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_title: Vec<String> = rows_title.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_title.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("title".to_string(), (vals_title, total_title));
            let page_size_lead = 10u64;
            let cur_page_lead = pages.get("lead").copied().unwrap_or(0);
            let count_stmt_lead = Query::select().expr(Expr::cust("COUNT(DISTINCT lead)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("lead")).is_not_null()).to_owned();
            let count_row_lead = match db.query_one(&count_stmt_lead).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.lead` : colonne introuvable en DB — {}", e); None } };
            let total_lead = count_row_lead.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_lead = Query::select().distinct().expr(Expr::cust("CAST(lead AS TEXT)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("lead")).is_not_null()).limit(page_size_lead).offset(cur_page_lead * page_size_lead).to_owned();
            let rows_lead = match db.query_all(&stmt_lead).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.lead` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_lead: Vec<String> = rows_lead.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_lead.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("lead".to_string(), (vals_lead, total_lead));
            let page_size_page_type = 10u64;
            let cur_page_page_type = pages.get("page_type").copied().unwrap_or(0);
            let count_stmt_page_type = Query::select().expr(Expr::cust("COUNT(DISTINCT page_type)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("page_type")).is_not_null()).to_owned();
            let count_row_page_type = match db.query_one(&count_stmt_page_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.page_type` : colonne introuvable en DB — {}", e); None } };
            let total_page_type = count_row_page_type.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_page_type = Query::select().distinct().expr(Expr::cust("CAST(page_type AS TEXT)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("page_type")).is_not_null()).limit(page_size_page_type).offset(cur_page_page_type * page_size_page_type).to_owned();
            let rows_page_type = match db.query_all(&stmt_page_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.page_type` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_page_type: Vec<String> = rows_page_type.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_page_type.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("page_type".to_string(), (vals_page_type, total_page_type));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(demo_page::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_page.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : demo_section ──
    let meta = AdminResource::new(
        "demo_section",
        "crate::entities::demo_section::Model",
        "AdminForm",
        "Sections",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = demo_section::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(DemoSectionAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = demo_section::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            demo_section::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = demo_section::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_section::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            demo_section::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            demo_section::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("page_id", "Page"), ("title", "Titre"), ("content", "Contenu"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("page_id", "Page", 10u64), ("title", "Titre", 10u64), ("content", "Contenu", 10u64), ("sort_order", "Ordre d'affichage", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_page_id = 10u64;
            let cur_page_page_id = pages.get("page_id").copied().unwrap_or(0);
            let count_stmt_page_id = Query::select().expr(Expr::cust("COUNT(DISTINCT page_id)")).from(Alias::new(demo_section::Entity.table_name())).and_where(Expr::col(Alias::new("page_id")).is_not_null()).to_owned();
            let count_row_page_id = match db.query_one(&count_stmt_page_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_section.page_id` : colonne introuvable en DB — {}", e); None } };
            let total_page_id = count_row_page_id.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_page_id = Query::select().distinct().expr(Expr::cust("CAST(page_id AS TEXT)")).from(Alias::new(demo_section::Entity.table_name())).and_where(Expr::col(Alias::new("page_id")).is_not_null()).limit(page_size_page_id).offset(cur_page_page_id * page_size_page_id).to_owned();
            let rows_page_id = match db.query_all(&stmt_page_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_section.page_id` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_page_id: Vec<String> = rows_page_id.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_page_id.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("page_id".to_string(), (vals_page_id, total_page_id));
            let page_size_title = 10u64;
            let cur_page_title = pages.get("title").copied().unwrap_or(0);
            let count_stmt_title = Query::select().expr(Expr::cust("COUNT(DISTINCT title)")).from(Alias::new(demo_section::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).to_owned();
            let count_row_title = match db.query_one(&count_stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_section.title` : colonne introuvable en DB — {}", e); None } };
            let total_title = count_row_title.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_title = Query::select().distinct().expr(Expr::cust("CAST(title AS TEXT)")).from(Alias::new(demo_section::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).limit(page_size_title).offset(cur_page_title * page_size_title).to_owned();
            let rows_title = match db.query_all(&stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_section.title` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_title: Vec<String> = rows_title.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_title.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("title".to_string(), (vals_title, total_title));
            let page_size_content = 10u64;
            let cur_page_content = pages.get("content").copied().unwrap_or(0);
            let count_stmt_content = Query::select().expr(Expr::cust("COUNT(DISTINCT content)")).from(Alias::new(demo_section::Entity.table_name())).and_where(Expr::col(Alias::new("content")).is_not_null()).to_owned();
            let count_row_content = match db.query_one(&count_stmt_content).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_section.content` : colonne introuvable en DB — {}", e); None } };
            let total_content = count_row_content.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_content = Query::select().distinct().expr(Expr::cust("CAST(content AS TEXT)")).from(Alias::new(demo_section::Entity.table_name())).and_where(Expr::col(Alias::new("content")).is_not_null()).limit(page_size_content).offset(cur_page_content * page_size_content).to_owned();
            let rows_content = match db.query_all(&stmt_content).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_section.content` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_content: Vec<String> = rows_content.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_content.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("content".to_string(), (vals_content, total_content));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(demo_section::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_section.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(demo_section::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `demo_section.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : code_example ──
    let meta = AdminResource::new(
        "code_example",
        "crate::entities::code_example::Model",
        "AdminForm",
        "Exemples de code",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = code_example::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(CodeExampleAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = code_example::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            code_example::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = code_example::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            code_example::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            code_example::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            code_example::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("page_id", "Page"), ("title", "Titre"), ("language", "Langage"), ("code", "Code"), ("context", "Contexte"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("page_id", "Page", 10u64), ("title", "Titre", 10u64), ("language", "Langage", 10u64), ("code", "Code", 10u64), ("context", "Contexte", 10u64), ("sort_order", "Ordre d'affichage", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_page_id = 10u64;
            let cur_page_page_id = pages.get("page_id").copied().unwrap_or(0);
            let count_stmt_page_id = Query::select().expr(Expr::cust("COUNT(DISTINCT page_id)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("page_id")).is_not_null()).to_owned();
            let count_row_page_id = match db.query_one(&count_stmt_page_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.page_id` : colonne introuvable en DB — {}", e); None } };
            let total_page_id = count_row_page_id.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_page_id = Query::select().distinct().expr(Expr::cust("CAST(page_id AS TEXT)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("page_id")).is_not_null()).limit(page_size_page_id).offset(cur_page_page_id * page_size_page_id).to_owned();
            let rows_page_id = match db.query_all(&stmt_page_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.page_id` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_page_id: Vec<String> = rows_page_id.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_page_id.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("page_id".to_string(), (vals_page_id, total_page_id));
            let page_size_title = 10u64;
            let cur_page_title = pages.get("title").copied().unwrap_or(0);
            let count_stmt_title = Query::select().expr(Expr::cust("COUNT(DISTINCT title)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).to_owned();
            let count_row_title = match db.query_one(&count_stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.title` : colonne introuvable en DB — {}", e); None } };
            let total_title = count_row_title.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_title = Query::select().distinct().expr(Expr::cust("CAST(title AS TEXT)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).limit(page_size_title).offset(cur_page_title * page_size_title).to_owned();
            let rows_title = match db.query_all(&stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.title` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_title: Vec<String> = rows_title.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_title.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("title".to_string(), (vals_title, total_title));
            let page_size_language = 10u64;
            let cur_page_language = pages.get("language").copied().unwrap_or(0);
            let count_stmt_language = Query::select().expr(Expr::cust("COUNT(DISTINCT language)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("language")).is_not_null()).to_owned();
            let count_row_language = match db.query_one(&count_stmt_language).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.language` : colonne introuvable en DB — {}", e); None } };
            let total_language = count_row_language.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_language = Query::select().distinct().expr(Expr::cust("CAST(language AS TEXT)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("language")).is_not_null()).limit(page_size_language).offset(cur_page_language * page_size_language).to_owned();
            let rows_language = match db.query_all(&stmt_language).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.language` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_language: Vec<String> = rows_language.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_language.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("language".to_string(), (vals_language, total_language));
            let page_size_code = 10u64;
            let cur_page_code = pages.get("code").copied().unwrap_or(0);
            let count_stmt_code = Query::select().expr(Expr::cust("COUNT(DISTINCT code)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("code")).is_not_null()).to_owned();
            let count_row_code = match db.query_one(&count_stmt_code).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.code` : colonne introuvable en DB — {}", e); None } };
            let total_code = count_row_code.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_code = Query::select().distinct().expr(Expr::cust("CAST(code AS TEXT)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("code")).is_not_null()).limit(page_size_code).offset(cur_page_code * page_size_code).to_owned();
            let rows_code = match db.query_all(&stmt_code).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.code` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_code: Vec<String> = rows_code.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_code.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("code".to_string(), (vals_code, total_code));
            let page_size_context = 10u64;
            let cur_page_context = pages.get("context").copied().unwrap_or(0);
            let count_stmt_context = Query::select().expr(Expr::cust("COUNT(DISTINCT context)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("context")).is_not_null()).to_owned();
            let count_row_context = match db.query_one(&count_stmt_context).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.context` : colonne introuvable en DB — {}", e); None } };
            let total_context = count_row_context.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_context = Query::select().distinct().expr(Expr::cust("CAST(context AS TEXT)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("context")).is_not_null()).limit(page_size_context).offset(cur_page_context * page_size_context).to_owned();
            let rows_context = match db.query_all(&stmt_context).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.context` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_context: Vec<String> = rows_context.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_context.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("context".to_string(), (vals_context, total_context));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(code_example::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `code_example.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : page_doc_link ──
    let meta = AdminResource::new(
        "page_doc_link",
        "crate::entities::page_doc_link::Model",
        "AdminForm",
        "Liens documentation",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = page_doc_link::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(PageDocLinkAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = page_doc_link::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            page_doc_link::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = page_doc_link::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            page_doc_link::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            page_doc_link::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            page_doc_link::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("page_id", "Page"), ("label", "Label"), ("url", "URL"), ("link_type", "Type"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("page_id", "Page", 10u64), ("label", "Label", 10u64), ("url", "URL", 10u64), ("link_type", "Type", 10u64), ("sort_order", "Ordre d'affichage", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_page_id = 10u64;
            let cur_page_page_id = pages.get("page_id").copied().unwrap_or(0);
            let count_stmt_page_id = Query::select().expr(Expr::cust("COUNT(DISTINCT page_id)")).from(Alias::new(page_doc_link::Entity.table_name())).and_where(Expr::col(Alias::new("page_id")).is_not_null()).to_owned();
            let count_row_page_id = match db.query_one(&count_stmt_page_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `page_doc_link.page_id` : colonne introuvable en DB — {}", e); None } };
            let total_page_id = count_row_page_id.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_page_id = Query::select().distinct().expr(Expr::cust("CAST(page_id AS TEXT)")).from(Alias::new(page_doc_link::Entity.table_name())).and_where(Expr::col(Alias::new("page_id")).is_not_null()).limit(page_size_page_id).offset(cur_page_page_id * page_size_page_id).to_owned();
            let rows_page_id = match db.query_all(&stmt_page_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `page_doc_link.page_id` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_page_id: Vec<String> = rows_page_id.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_page_id.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("page_id".to_string(), (vals_page_id, total_page_id));
            let page_size_label = 10u64;
            let cur_page_label = pages.get("label").copied().unwrap_or(0);
            let count_stmt_label = Query::select().expr(Expr::cust("COUNT(DISTINCT label)")).from(Alias::new(page_doc_link::Entity.table_name())).and_where(Expr::col(Alias::new("label")).is_not_null()).to_owned();
            let count_row_label = match db.query_one(&count_stmt_label).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `page_doc_link.label` : colonne introuvable en DB — {}", e); None } };
            let total_label = count_row_label.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_label = Query::select().distinct().expr(Expr::cust("CAST(label AS TEXT)")).from(Alias::new(page_doc_link::Entity.table_name())).and_where(Expr::col(Alias::new("label")).is_not_null()).limit(page_size_label).offset(cur_page_label * page_size_label).to_owned();
            let rows_label = match db.query_all(&stmt_label).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `page_doc_link.label` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_label: Vec<String> = rows_label.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_label.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("label".to_string(), (vals_label, total_label));
            let page_size_url = 10u64;
            let cur_page_url = pages.get("url").copied().unwrap_or(0);
            let count_stmt_url = Query::select().expr(Expr::cust("COUNT(DISTINCT url)")).from(Alias::new(page_doc_link::Entity.table_name())).and_where(Expr::col(Alias::new("url")).is_not_null()).to_owned();
            let count_row_url = match db.query_one(&count_stmt_url).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `page_doc_link.url` : colonne introuvable en DB — {}", e); None } };
            let total_url = count_row_url.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_url = Query::select().distinct().expr(Expr::cust("CAST(url AS TEXT)")).from(Alias::new(page_doc_link::Entity.table_name())).and_where(Expr::col(Alias::new("url")).is_not_null()).limit(page_size_url).offset(cur_page_url * page_size_url).to_owned();
            let rows_url = match db.query_all(&stmt_url).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `page_doc_link.url` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_url: Vec<String> = rows_url.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_url.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("url".to_string(), (vals_url, total_url));
            let page_size_link_type = 10u64;
            let cur_page_link_type = pages.get("link_type").copied().unwrap_or(0);
            let count_stmt_link_type = Query::select().expr(Expr::cust("COUNT(DISTINCT link_type)")).from(Alias::new(page_doc_link::Entity.table_name())).and_where(Expr::col(Alias::new("link_type")).is_not_null()).to_owned();
            let count_row_link_type = match db.query_one(&count_stmt_link_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `page_doc_link.link_type` : colonne introuvable en DB — {}", e); None } };
            let total_link_type = count_row_link_type.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_link_type = Query::select().distinct().expr(Expr::cust("CAST(link_type AS TEXT)")).from(Alias::new(page_doc_link::Entity.table_name())).and_where(Expr::col(Alias::new("link_type")).is_not_null()).limit(page_size_link_type).offset(cur_page_link_type * page_size_link_type).to_owned();
            let rows_link_type = match db.query_all(&stmt_link_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `page_doc_link.link_type` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_link_type: Vec<String> = rows_link_type.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_link_type.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("link_type".to_string(), (vals_link_type, total_link_type));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(page_doc_link::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `page_doc_link.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(page_doc_link::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `page_doc_link.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : form_field ──
    let meta = AdminResource::new(
        "form_field",
        "crate::entities::form_field::Model",
        "AdminForm",
        "Champs formulaire",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = form_field::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(FormFieldAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = form_field::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            form_field::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = form_field::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            form_field::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            form_field::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            form_field::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("page_id", "Page"), ("name", "Nom"), ("field_type", "Type"), ("description", "Description"), ("example", "Exemple"), ("html_preview", "Aperçu HTML"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("page_id", "Page", 10u64), ("name", "Nom", 10u64), ("field_type", "Type", 10u64), ("description", "Description", 10u64), ("example", "Exemple", 10u64), ("html_preview", "Aperçu HTML", 10u64), ("sort_order", "Ordre d'affichage", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_page_id = 10u64;
            let cur_page_page_id = pages.get("page_id").copied().unwrap_or(0);
            let count_stmt_page_id = Query::select().expr(Expr::cust("COUNT(DISTINCT page_id)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("page_id")).is_not_null()).to_owned();
            let count_row_page_id = match db.query_one(&count_stmt_page_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.page_id` : colonne introuvable en DB — {}", e); None } };
            let total_page_id = count_row_page_id.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_page_id = Query::select().distinct().expr(Expr::cust("CAST(page_id AS TEXT)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("page_id")).is_not_null()).limit(page_size_page_id).offset(cur_page_page_id * page_size_page_id).to_owned();
            let rows_page_id = match db.query_all(&stmt_page_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.page_id` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_page_id: Vec<String> = rows_page_id.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_page_id.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("page_id".to_string(), (vals_page_id, total_page_id));
            let page_size_name = 10u64;
            let cur_page_name = pages.get("name").copied().unwrap_or(0);
            let count_stmt_name = Query::select().expr(Expr::cust("COUNT(DISTINCT name)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("name")).is_not_null()).to_owned();
            let count_row_name = match db.query_one(&count_stmt_name).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.name` : colonne introuvable en DB — {}", e); None } };
            let total_name = count_row_name.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_name = Query::select().distinct().expr(Expr::cust("CAST(name AS TEXT)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("name")).is_not_null()).limit(page_size_name).offset(cur_page_name * page_size_name).to_owned();
            let rows_name = match db.query_all(&stmt_name).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.name` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_name: Vec<String> = rows_name.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_name.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("name".to_string(), (vals_name, total_name));
            let page_size_field_type = 10u64;
            let cur_page_field_type = pages.get("field_type").copied().unwrap_or(0);
            let count_stmt_field_type = Query::select().expr(Expr::cust("COUNT(DISTINCT field_type)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("field_type")).is_not_null()).to_owned();
            let count_row_field_type = match db.query_one(&count_stmt_field_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.field_type` : colonne introuvable en DB — {}", e); None } };
            let total_field_type = count_row_field_type.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_field_type = Query::select().distinct().expr(Expr::cust("CAST(field_type AS TEXT)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("field_type")).is_not_null()).limit(page_size_field_type).offset(cur_page_field_type * page_size_field_type).to_owned();
            let rows_field_type = match db.query_all(&stmt_field_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.field_type` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_field_type: Vec<String> = rows_field_type.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_field_type.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("field_type".to_string(), (vals_field_type, total_field_type));
            let page_size_description = 10u64;
            let cur_page_description = pages.get("description").copied().unwrap_or(0);
            let count_stmt_description = Query::select().expr(Expr::cust("COUNT(DISTINCT description)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("description")).is_not_null()).to_owned();
            let count_row_description = match db.query_one(&count_stmt_description).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.description` : colonne introuvable en DB — {}", e); None } };
            let total_description = count_row_description.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_description = Query::select().distinct().expr(Expr::cust("CAST(description AS TEXT)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("description")).is_not_null()).limit(page_size_description).offset(cur_page_description * page_size_description).to_owned();
            let rows_description = match db.query_all(&stmt_description).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.description` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_description: Vec<String> = rows_description.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_description.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("description".to_string(), (vals_description, total_description));
            let page_size_example = 10u64;
            let cur_page_example = pages.get("example").copied().unwrap_or(0);
            let count_stmt_example = Query::select().expr(Expr::cust("COUNT(DISTINCT example)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("example")).is_not_null()).to_owned();
            let count_row_example = match db.query_one(&count_stmt_example).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.example` : colonne introuvable en DB — {}", e); None } };
            let total_example = count_row_example.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_example = Query::select().distinct().expr(Expr::cust("CAST(example AS TEXT)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("example")).is_not_null()).limit(page_size_example).offset(cur_page_example * page_size_example).to_owned();
            let rows_example = match db.query_all(&stmt_example).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.example` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_example: Vec<String> = rows_example.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_example.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("example".to_string(), (vals_example, total_example));
            let page_size_html_preview = 10u64;
            let cur_page_html_preview = pages.get("html_preview").copied().unwrap_or(0);
            let count_stmt_html_preview = Query::select().expr(Expr::cust("COUNT(DISTINCT html_preview)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("html_preview")).is_not_null()).to_owned();
            let count_row_html_preview = match db.query_one(&count_stmt_html_preview).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.html_preview` : colonne introuvable en DB — {}", e); None } };
            let total_html_preview = count_row_html_preview.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_html_preview = Query::select().distinct().expr(Expr::cust("CAST(html_preview AS TEXT)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("html_preview")).is_not_null()).limit(page_size_html_preview).offset(cur_page_html_preview * page_size_html_preview).to_owned();
            let rows_html_preview = match db.query_all(&stmt_html_preview).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.html_preview` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_html_preview: Vec<String> = rows_html_preview.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_html_preview.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("html_preview".to_string(), (vals_html_preview, total_html_preview));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(form_field::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `form_field.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : doc_section ──
    let meta = AdminResource::new(
        "doc_section",
        "crate::entities::doc_section::Model",
        "AdminForm",
        "Doc — Sections",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = doc_section::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(DocSectionAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = doc_section::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            doc_section::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = doc_section::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_section::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            doc_section::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_section::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("slug", "Slug"), ("lang", "Langue"), ("title", "Titre"), ("theme", "Thème"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("lang", "Langue", 10u64), ("theme", "Thème", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_lang = 10u64;
            let cur_page_lang = pages.get("lang").copied().unwrap_or(0);
            let count_stmt_lang = Query::select().expr(Expr::cust("COUNT(DISTINCT lang)")).from(Alias::new(doc_section::Entity.table_name())).and_where(Expr::col(Alias::new("lang")).is_not_null()).to_owned();
            let count_row_lang = match db.query_one(&count_stmt_lang).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_section.lang` : colonne introuvable en DB — {}", e); None } };
            let total_lang = count_row_lang.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_lang = Query::select().distinct().expr(Expr::cust("CAST(lang AS TEXT)")).from(Alias::new(doc_section::Entity.table_name())).and_where(Expr::col(Alias::new("lang")).is_not_null()).limit(page_size_lang).offset(cur_page_lang * page_size_lang).to_owned();
            let rows_lang = match db.query_all(&stmt_lang).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_section.lang` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_lang: Vec<String> = rows_lang.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_lang.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("lang".to_string(), (vals_lang, total_lang));
            let page_size_theme = 10u64;
            let cur_page_theme = pages.get("theme").copied().unwrap_or(0);
            let count_stmt_theme = Query::select().expr(Expr::cust("COUNT(DISTINCT theme)")).from(Alias::new(doc_section::Entity.table_name())).and_where(Expr::col(Alias::new("theme")).is_not_null()).to_owned();
            let count_row_theme = match db.query_one(&count_stmt_theme).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_section.theme` : colonne introuvable en DB — {}", e); None } };
            let total_theme = count_row_theme.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_theme = Query::select().distinct().expr(Expr::cust("CAST(theme AS TEXT)")).from(Alias::new(doc_section::Entity.table_name())).and_where(Expr::col(Alias::new("theme")).is_not_null()).limit(page_size_theme).offset(cur_page_theme * page_size_theme).to_owned();
            let rows_theme = match db.query_all(&stmt_theme).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_section.theme` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_theme: Vec<String> = rows_theme.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_theme.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("theme".to_string(), (vals_theme, total_theme));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : doc_page ──
    let meta = AdminResource::new(
        "doc_page",
        "crate::entities::doc_page::Model",
        "AdminForm",
        "Doc — Pages",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = doc_page::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(DocPageAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = doc_page::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            doc_page::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = doc_page::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_page::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            doc_page::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_page::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("section_id", "Section"), ("slug", "Slug"), ("lang", "Langue"), ("title", "Titre"), ("lead", "Lead"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("section_id", "Section", 10u64), ("slug", "Slug", 10u64), ("lang", "Langue", 10u64), ("title", "Titre", 10u64), ("lead", "Lead", 10u64), ("sort_order", "Ordre d'affichage", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_section_id = 10u64;
            let cur_page_section_id = pages.get("section_id").copied().unwrap_or(0);
            let count_stmt_section_id = Query::select().expr(Expr::cust("COUNT(DISTINCT section_id)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("section_id")).is_not_null()).to_owned();
            let count_row_section_id = match db.query_one(&count_stmt_section_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.section_id` : colonne introuvable en DB — {}", e); None } };
            let total_section_id = count_row_section_id.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_section_id = Query::select().distinct().expr(Expr::cust("CAST(section_id AS TEXT)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("section_id")).is_not_null()).limit(page_size_section_id).offset(cur_page_section_id * page_size_section_id).to_owned();
            let rows_section_id = match db.query_all(&stmt_section_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.section_id` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_section_id: Vec<String> = rows_section_id.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_section_id.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("section_id".to_string(), (vals_section_id, total_section_id));
            let page_size_slug = 10u64;
            let cur_page_slug = pages.get("slug").copied().unwrap_or(0);
            let count_stmt_slug = Query::select().expr(Expr::cust("COUNT(DISTINCT slug)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("slug")).is_not_null()).to_owned();
            let count_row_slug = match db.query_one(&count_stmt_slug).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.slug` : colonne introuvable en DB — {}", e); None } };
            let total_slug = count_row_slug.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_slug = Query::select().distinct().expr(Expr::cust("CAST(slug AS TEXT)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("slug")).is_not_null()).limit(page_size_slug).offset(cur_page_slug * page_size_slug).to_owned();
            let rows_slug = match db.query_all(&stmt_slug).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.slug` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_slug: Vec<String> = rows_slug.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_slug.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("slug".to_string(), (vals_slug, total_slug));
            let page_size_lang = 10u64;
            let cur_page_lang = pages.get("lang").copied().unwrap_or(0);
            let count_stmt_lang = Query::select().expr(Expr::cust("COUNT(DISTINCT lang)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("lang")).is_not_null()).to_owned();
            let count_row_lang = match db.query_one(&count_stmt_lang).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.lang` : colonne introuvable en DB — {}", e); None } };
            let total_lang = count_row_lang.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_lang = Query::select().distinct().expr(Expr::cust("CAST(lang AS TEXT)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("lang")).is_not_null()).limit(page_size_lang).offset(cur_page_lang * page_size_lang).to_owned();
            let rows_lang = match db.query_all(&stmt_lang).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.lang` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_lang: Vec<String> = rows_lang.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_lang.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("lang".to_string(), (vals_lang, total_lang));
            let page_size_title = 10u64;
            let cur_page_title = pages.get("title").copied().unwrap_or(0);
            let count_stmt_title = Query::select().expr(Expr::cust("COUNT(DISTINCT title)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).to_owned();
            let count_row_title = match db.query_one(&count_stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.title` : colonne introuvable en DB — {}", e); None } };
            let total_title = count_row_title.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_title = Query::select().distinct().expr(Expr::cust("CAST(title AS TEXT)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("title")).is_not_null()).limit(page_size_title).offset(cur_page_title * page_size_title).to_owned();
            let rows_title = match db.query_all(&stmt_title).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.title` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_title: Vec<String> = rows_title.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_title.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("title".to_string(), (vals_title, total_title));
            let page_size_lead = 10u64;
            let cur_page_lead = pages.get("lead").copied().unwrap_or(0);
            let count_stmt_lead = Query::select().expr(Expr::cust("COUNT(DISTINCT lead)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("lead")).is_not_null()).to_owned();
            let count_row_lead = match db.query_one(&count_stmt_lead).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.lead` : colonne introuvable en DB — {}", e); None } };
            let total_lead = count_row_lead.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_lead = Query::select().distinct().expr(Expr::cust("CAST(lead AS TEXT)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("lead")).is_not_null()).limit(page_size_lead).offset(cur_page_lead * page_size_lead).to_owned();
            let rows_lead = match db.query_all(&stmt_lead).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.lead` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_lead: Vec<String> = rows_lead.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_lead.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("lead".to_string(), (vals_lead, total_lead));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(doc_page::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_page.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : doc_block ──
    let meta = AdminResource::new(
        "doc_block",
        "crate::entities::doc_block::Model",
        "AdminForm",
        "Doc — Blocs",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = doc_block::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(DocBlockAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = doc_block::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            doc_block::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = doc_block::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_block::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            doc_block::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            doc_block::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("page_id", "Page"), ("content", "Contenu"), ("block_type", "Type"), ("heading", "En-tête"), ("sort_order", "Ordre")]).list_filter(vec![("page_id", "page", 10u64), ("heading", "En-tête", 10u64), ("content", "Contenu", 10u64), ("block_type", "type", 10u64), ("sort_order", "Ordre", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_page_id = 10u64;
            let cur_page_page_id = pages.get("page_id").copied().unwrap_or(0);
            let count_stmt_page_id = Query::select().expr(Expr::cust("COUNT(DISTINCT page_id)")).from(Alias::new(doc_block::Entity.table_name())).and_where(Expr::col(Alias::new("page_id")).is_not_null()).to_owned();
            let count_row_page_id = match db.query_one(&count_stmt_page_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_block.page_id` : colonne introuvable en DB — {}", e); None } };
            let total_page_id = count_row_page_id.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_page_id = Query::select().distinct().expr(Expr::cust("CAST(page_id AS TEXT)")).from(Alias::new(doc_block::Entity.table_name())).and_where(Expr::col(Alias::new("page_id")).is_not_null()).limit(page_size_page_id).offset(cur_page_page_id * page_size_page_id).to_owned();
            let rows_page_id = match db.query_all(&stmt_page_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_block.page_id` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_page_id: Vec<String> = rows_page_id.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_page_id.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("page_id".to_string(), (vals_page_id, total_page_id));
            let page_size_heading = 10u64;
            let cur_page_heading = pages.get("heading").copied().unwrap_or(0);
            let count_stmt_heading = Query::select().expr(Expr::cust("COUNT(DISTINCT heading)")).from(Alias::new(doc_block::Entity.table_name())).and_where(Expr::col(Alias::new("heading")).is_not_null()).to_owned();
            let count_row_heading = match db.query_one(&count_stmt_heading).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_block.heading` : colonne introuvable en DB — {}", e); None } };
            let total_heading = count_row_heading.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_heading = Query::select().distinct().expr(Expr::cust("CAST(heading AS TEXT)")).from(Alias::new(doc_block::Entity.table_name())).and_where(Expr::col(Alias::new("heading")).is_not_null()).limit(page_size_heading).offset(cur_page_heading * page_size_heading).to_owned();
            let rows_heading = match db.query_all(&stmt_heading).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_block.heading` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_heading: Vec<String> = rows_heading.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_heading.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("heading".to_string(), (vals_heading, total_heading));
            let page_size_content = 10u64;
            let cur_page_content = pages.get("content").copied().unwrap_or(0);
            let count_stmt_content = Query::select().expr(Expr::cust("COUNT(DISTINCT content)")).from(Alias::new(doc_block::Entity.table_name())).and_where(Expr::col(Alias::new("content")).is_not_null()).to_owned();
            let count_row_content = match db.query_one(&count_stmt_content).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_block.content` : colonne introuvable en DB — {}", e); None } };
            let total_content = count_row_content.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_content = Query::select().distinct().expr(Expr::cust("CAST(content AS TEXT)")).from(Alias::new(doc_block::Entity.table_name())).and_where(Expr::col(Alias::new("content")).is_not_null()).limit(page_size_content).offset(cur_page_content * page_size_content).to_owned();
            let rows_content = match db.query_all(&stmt_content).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_block.content` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_content: Vec<String> = rows_content.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_content.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("content".to_string(), (vals_content, total_content));
            let page_size_block_type = 10u64;
            let cur_page_block_type = pages.get("block_type").copied().unwrap_or(0);
            let count_stmt_block_type = Query::select().expr(Expr::cust("COUNT(DISTINCT block_type)")).from(Alias::new(doc_block::Entity.table_name())).and_where(Expr::col(Alias::new("block_type")).is_not_null()).to_owned();
            let count_row_block_type = match db.query_one(&count_stmt_block_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_block.block_type` : colonne introuvable en DB — {}", e); None } };
            let total_block_type = count_row_block_type.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_block_type = Query::select().distinct().expr(Expr::cust("CAST(block_type AS TEXT)")).from(Alias::new(doc_block::Entity.table_name())).and_where(Expr::col(Alias::new("block_type")).is_not_null()).limit(page_size_block_type).offset(cur_page_block_type * page_size_block_type).to_owned();
            let rows_block_type = match db.query_all(&stmt_block_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_block.block_type` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_block_type: Vec<String> = rows_block_type.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_block_type.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("block_type".to_string(), (vals_block_type, total_block_type));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(doc_block::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_block.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(doc_block::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `doc_block.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : site_config ──
    let meta = AdminResource::new(
        "site_config",
        "crate::entities::site_config::Model",
        "AdminForm",
        "Configuration site",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = site_config::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(SiteConfigAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = site_config::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            site_config::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = site_config::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            site_config::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            site_config::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            site_config::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
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
    );

    // ── Ressource : cour ──
    let meta = AdminResource::new(
        "cour",
        "crate::entities::cour::Model",
        "AdminForm",
        "Cours",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = cour::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(CourAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = cour::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            cour::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = cour::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            cour::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            cour::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            cour::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("slug", "Slug"), ("lang", "Langue"), ("titre", "Titre"), ("theme", "Thème"), ("difficulte", "Difficulté"), ("ordre", "Ordre"), ("sort_order", "Ordre d'affichage")]).list_filter(vec![("slug", "Slug", 10u64), ("lang", "Langue", 10u64), ("titre", "Titre", 10u64), ("theme", "Thème", 10u64), ("difficulte", "Difficulté", 10u64), ("ordre", "Ordre", 10u64), ("sort_order", "Ordre d'affichage", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_slug = 10u64;
            let cur_page_slug = pages.get("slug").copied().unwrap_or(0);
            let count_stmt_slug = Query::select().expr(Expr::cust("COUNT(DISTINCT slug)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("slug")).is_not_null()).to_owned();
            let count_row_slug = match db.query_one(&count_stmt_slug).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.slug` : colonne introuvable en DB — {}", e); None } };
            let total_slug = count_row_slug.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_slug = Query::select().distinct().expr(Expr::cust("CAST(slug AS TEXT)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("slug")).is_not_null()).limit(page_size_slug).offset(cur_page_slug * page_size_slug).to_owned();
            let rows_slug = match db.query_all(&stmt_slug).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.slug` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_slug: Vec<String> = rows_slug.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_slug.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("slug".to_string(), (vals_slug, total_slug));
            let page_size_lang = 10u64;
            let cur_page_lang = pages.get("lang").copied().unwrap_or(0);
            let count_stmt_lang = Query::select().expr(Expr::cust("COUNT(DISTINCT lang)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("lang")).is_not_null()).to_owned();
            let count_row_lang = match db.query_one(&count_stmt_lang).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.lang` : colonne introuvable en DB — {}", e); None } };
            let total_lang = count_row_lang.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_lang = Query::select().distinct().expr(Expr::cust("CAST(lang AS TEXT)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("lang")).is_not_null()).limit(page_size_lang).offset(cur_page_lang * page_size_lang).to_owned();
            let rows_lang = match db.query_all(&stmt_lang).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.lang` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_lang: Vec<String> = rows_lang.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_lang.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("lang".to_string(), (vals_lang, total_lang));
            let page_size_titre = 10u64;
            let cur_page_titre = pages.get("titre").copied().unwrap_or(0);
            let count_stmt_titre = Query::select().expr(Expr::cust("COUNT(DISTINCT titre)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("titre")).is_not_null()).to_owned();
            let count_row_titre = match db.query_one(&count_stmt_titre).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.titre` : colonne introuvable en DB — {}", e); None } };
            let total_titre = count_row_titre.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_titre = Query::select().distinct().expr(Expr::cust("CAST(titre AS TEXT)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("titre")).is_not_null()).limit(page_size_titre).offset(cur_page_titre * page_size_titre).to_owned();
            let rows_titre = match db.query_all(&stmt_titre).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.titre` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_titre: Vec<String> = rows_titre.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_titre.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("titre".to_string(), (vals_titre, total_titre));
            let page_size_theme = 10u64;
            let cur_page_theme = pages.get("theme").copied().unwrap_or(0);
            let count_stmt_theme = Query::select().expr(Expr::cust("COUNT(DISTINCT theme)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("theme")).is_not_null()).to_owned();
            let count_row_theme = match db.query_one(&count_stmt_theme).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.theme` : colonne introuvable en DB — {}", e); None } };
            let total_theme = count_row_theme.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_theme = Query::select().distinct().expr(Expr::cust("CAST(theme AS TEXT)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("theme")).is_not_null()).limit(page_size_theme).offset(cur_page_theme * page_size_theme).to_owned();
            let rows_theme = match db.query_all(&stmt_theme).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.theme` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_theme: Vec<String> = rows_theme.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_theme.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("theme".to_string(), (vals_theme, total_theme));
            let page_size_difficulte = 10u64;
            let cur_page_difficulte = pages.get("difficulte").copied().unwrap_or(0);
            let count_stmt_difficulte = Query::select().expr(Expr::cust("COUNT(DISTINCT difficulte)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("difficulte")).is_not_null()).to_owned();
            let count_row_difficulte = match db.query_one(&count_stmt_difficulte).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.difficulte` : colonne introuvable en DB — {}", e); None } };
            let total_difficulte = count_row_difficulte.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_difficulte = Query::select().distinct().expr(Expr::cust("CAST(difficulte AS TEXT)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("difficulte")).is_not_null()).limit(page_size_difficulte).offset(cur_page_difficulte * page_size_difficulte).to_owned();
            let rows_difficulte = match db.query_all(&stmt_difficulte).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.difficulte` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_difficulte: Vec<String> = rows_difficulte.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_difficulte.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("difficulte".to_string(), (vals_difficulte, total_difficulte));
            let page_size_ordre = 10u64;
            let cur_page_ordre = pages.get("ordre").copied().unwrap_or(0);
            let count_stmt_ordre = Query::select().expr(Expr::cust("COUNT(DISTINCT ordre)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("ordre")).is_not_null()).to_owned();
            let count_row_ordre = match db.query_one(&count_stmt_ordre).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.ordre` : colonne introuvable en DB — {}", e); None } };
            let total_ordre = count_row_ordre.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_ordre = Query::select().distinct().expr(Expr::cust("CAST(ordre AS TEXT)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("ordre")).is_not_null()).limit(page_size_ordre).offset(cur_page_ordre * page_size_ordre).to_owned();
            let rows_ordre = match db.query_all(&stmt_ordre).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.ordre` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_ordre: Vec<String> = rows_ordre.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_ordre.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("ordre".to_string(), (vals_ordre, total_ordre));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(cour::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : chapitre ──
    let meta = AdminResource::new(
        "chapitre",
        "crate::entities::chapitre::Model",
        "AdminForm",
        "Chapitres",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = chapitre::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(ChapitreAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = chapitre::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            chapitre::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = chapitre::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            chapitre::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            chapitre::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            chapitre::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("cour_id", "Cours"), ("slug", "Slug"), ("titre", "Titre"), ("ordre", "Ordre")]).list_filter(vec![("cour_id", "Cours", 10u64), ("slug", "Slug", 10u64), ("titre", "Titre", 10u64), ("sort_order", "Ordre", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_cour_id = 10u64;
            let cur_page_cour_id = pages.get("cour_id").copied().unwrap_or(0);
            let count_stmt_cour_id = Query::select().expr(Expr::cust("COUNT(DISTINCT cour_id)")).from(Alias::new(chapitre::Entity.table_name())).and_where(Expr::col(Alias::new("cour_id")).is_not_null()).to_owned();
            let count_row_cour_id = match db.query_one(&count_stmt_cour_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `chapitre.cour_id` : colonne introuvable en DB — {}", e); None } };
            let total_cour_id = count_row_cour_id.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_cour_id = Query::select().distinct().expr(Expr::cust("CAST(cour_id AS TEXT)")).from(Alias::new(chapitre::Entity.table_name())).and_where(Expr::col(Alias::new("cour_id")).is_not_null()).limit(page_size_cour_id).offset(cur_page_cour_id * page_size_cour_id).to_owned();
            let rows_cour_id = match db.query_all(&stmt_cour_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `chapitre.cour_id` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_cour_id: Vec<String> = rows_cour_id.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_cour_id.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("cour_id".to_string(), (vals_cour_id, total_cour_id));
            let page_size_slug = 10u64;
            let cur_page_slug = pages.get("slug").copied().unwrap_or(0);
            let count_stmt_slug = Query::select().expr(Expr::cust("COUNT(DISTINCT slug)")).from(Alias::new(chapitre::Entity.table_name())).and_where(Expr::col(Alias::new("slug")).is_not_null()).to_owned();
            let count_row_slug = match db.query_one(&count_stmt_slug).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `chapitre.slug` : colonne introuvable en DB — {}", e); None } };
            let total_slug = count_row_slug.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_slug = Query::select().distinct().expr(Expr::cust("CAST(slug AS TEXT)")).from(Alias::new(chapitre::Entity.table_name())).and_where(Expr::col(Alias::new("slug")).is_not_null()).limit(page_size_slug).offset(cur_page_slug * page_size_slug).to_owned();
            let rows_slug = match db.query_all(&stmt_slug).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `chapitre.slug` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_slug: Vec<String> = rows_slug.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_slug.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("slug".to_string(), (vals_slug, total_slug));
            let page_size_titre = 10u64;
            let cur_page_titre = pages.get("titre").copied().unwrap_or(0);
            let count_stmt_titre = Query::select().expr(Expr::cust("COUNT(DISTINCT titre)")).from(Alias::new(chapitre::Entity.table_name())).and_where(Expr::col(Alias::new("titre")).is_not_null()).to_owned();
            let count_row_titre = match db.query_one(&count_stmt_titre).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `chapitre.titre` : colonne introuvable en DB — {}", e); None } };
            let total_titre = count_row_titre.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_titre = Query::select().distinct().expr(Expr::cust("CAST(titre AS TEXT)")).from(Alias::new(chapitre::Entity.table_name())).and_where(Expr::col(Alias::new("titre")).is_not_null()).limit(page_size_titre).offset(cur_page_titre * page_size_titre).to_owned();
            let rows_titre = match db.query_all(&stmt_titre).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `chapitre.titre` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_titre: Vec<String> = rows_titre.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_titre.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("titre".to_string(), (vals_titre, total_titre));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(chapitre::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `chapitre.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(chapitre::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `chapitre.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : cour_block ──
    let meta = AdminResource::new(
        "cour_block",
        "crate::entities::cour_block::Model",
        "AdminForm",
        "Cours — Blocs",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = cour_block::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(CourBlockAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = cour_block::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            cour_block::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = cour_block::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            cour_block::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            cour_block::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            cour_block::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("chapitre_id", "Chapitre"), ("block_type", "Type"), ("heading", "En-tête"), ("sort_order", "Ordre")]).list_filter(vec![("chapitre_id", "Chapitre", 10u64), ("block_type", "Type", 10u64), ("heading", "En-tête", 10u64), ("sort_order", "Ordre", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_chapitre_id = 10u64;
            let cur_page_chapitre_id = pages.get("chapitre_id").copied().unwrap_or(0);
            let count_stmt_chapitre_id = Query::select().expr(Expr::cust("COUNT(DISTINCT chapitre_id)")).from(Alias::new(cour_block::Entity.table_name())).and_where(Expr::col(Alias::new("chapitre_id")).is_not_null()).to_owned();
            let count_row_chapitre_id = match db.query_one(&count_stmt_chapitre_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour_block.chapitre_id` : colonne introuvable en DB — {}", e); None } };
            let total_chapitre_id = count_row_chapitre_id.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_chapitre_id = Query::select().distinct().expr(Expr::cust("CAST(chapitre_id AS TEXT)")).from(Alias::new(cour_block::Entity.table_name())).and_where(Expr::col(Alias::new("chapitre_id")).is_not_null()).limit(page_size_chapitre_id).offset(cur_page_chapitre_id * page_size_chapitre_id).to_owned();
            let rows_chapitre_id = match db.query_all(&stmt_chapitre_id).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour_block.chapitre_id` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_chapitre_id: Vec<String> = rows_chapitre_id.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_chapitre_id.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("chapitre_id".to_string(), (vals_chapitre_id, total_chapitre_id));
            let page_size_block_type = 10u64;
            let cur_page_block_type = pages.get("block_type").copied().unwrap_or(0);
            let count_stmt_block_type = Query::select().expr(Expr::cust("COUNT(DISTINCT block_type)")).from(Alias::new(cour_block::Entity.table_name())).and_where(Expr::col(Alias::new("block_type")).is_not_null()).to_owned();
            let count_row_block_type = match db.query_one(&count_stmt_block_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour_block.block_type` : colonne introuvable en DB — {}", e); None } };
            let total_block_type = count_row_block_type.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_block_type = Query::select().distinct().expr(Expr::cust("CAST(block_type AS TEXT)")).from(Alias::new(cour_block::Entity.table_name())).and_where(Expr::col(Alias::new("block_type")).is_not_null()).limit(page_size_block_type).offset(cur_page_block_type * page_size_block_type).to_owned();
            let rows_block_type = match db.query_all(&stmt_block_type).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour_block.block_type` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_block_type: Vec<String> = rows_block_type.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_block_type.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("block_type".to_string(), (vals_block_type, total_block_type));
            let page_size_heading = 10u64;
            let cur_page_heading = pages.get("heading").copied().unwrap_or(0);
            let count_stmt_heading = Query::select().expr(Expr::cust("COUNT(DISTINCT heading)")).from(Alias::new(cour_block::Entity.table_name())).and_where(Expr::col(Alias::new("heading")).is_not_null()).to_owned();
            let count_row_heading = match db.query_one(&count_stmt_heading).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour_block.heading` : colonne introuvable en DB — {}", e); None } };
            let total_heading = count_row_heading.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_heading = Query::select().distinct().expr(Expr::cust("CAST(heading AS TEXT)")).from(Alias::new(cour_block::Entity.table_name())).and_where(Expr::col(Alias::new("heading")).is_not_null()).limit(page_size_heading).offset(cur_page_heading * page_size_heading).to_owned();
            let rows_heading = match db.query_all(&stmt_heading).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour_block.heading` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_heading: Vec<String> = rows_heading.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_heading.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("heading".to_string(), (vals_heading, total_heading));
            let page_size_sort_order = 10u64;
            let cur_page_sort_order = pages.get("sort_order").copied().unwrap_or(0);
            let count_stmt_sort_order = Query::select().expr(Expr::cust("COUNT(DISTINCT sort_order)")).from(Alias::new(cour_block::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).to_owned();
            let count_row_sort_order = match db.query_one(&count_stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour_block.sort_order` : colonne introuvable en DB — {}", e); None } };
            let total_sort_order = count_row_sort_order.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_sort_order = Query::select().distinct().expr(Expr::cust("CAST(sort_order AS TEXT)")).from(Alias::new(cour_block::Entity.table_name())).and_where(Expr::col(Alias::new("sort_order")).is_not_null()).limit(page_size_sort_order).offset(cur_page_sort_order * page_size_sort_order).to_owned();
            let rows_sort_order = match db.query_all(&stmt_sort_order).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `cour_block.sort_order` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_sort_order: Vec<String> = rows_sort_order.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_sort_order.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("sort_order".to_string(), (vals_sort_order, total_sort_order));
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
            .with_filter_fn(filter_fn)
    );

    // ── Ressource : runique_release ──
    let meta = AdminResource::new(
        "runique_release",
        "crate::entities::runique_release::Model",
        "AdminForm",
        "Releases Runique",
        vec!["admin".to_string()],
    );
    let form_builder: FormBuilder = Arc::new(|data: StrMap, tera: ATera, csrf: String, method: Method| {
        Box::pin(async move {
            let form = runique_release::AdminForm::build_with_data(&data, tera, &csrf, method).await;
            Box::new(RuniqueReleaseAdminFormDynWrapper(form)) as Box<dyn DynForm>
        })
    });

    let list_fn: ListFn = Arc::new(|db: ADb, params: ListParams| {
        Box::pin(async move {
            use sea_orm::{QueryFilter, sea_query::{Alias, Expr, Order}};
            let mut query = runique_release::Entity::find();
            if let Some(ref col) = params.sort_by {
                let order = if params.sort_dir == SortDir::Desc { Order::Desc } else { Order::Asc };
                query = query.order_by(Expr::col(Alias::new(col.as_str())), order);
            }
            for (col, val) in &params.column_filters {
                let escaped = val.replace('\'', "''");
                query = query.filter(Expr::cust(format!("CAST({} AS TEXT) = '{}'", col, escaped)));
            }
            let rows = query.offset(params.offset).limit(params.limit).all(&*db).await?;
            Ok(rows.into_iter()
                .map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null))
                .collect())
        })
    });

    let count_fn: CountFn = Arc::new(|db: ADb, _search: Option<String>| {
        Box::pin(async move {
            runique_release::Entity::find().count(&*db).await
        })
    });

    let get_fn: GetFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            let row = runique_release::Entity::find_by_id(id).one(&*db).await?;
            Ok(row.map(|r| serde_json::to_value(r).unwrap_or(serde_json::Value::Null)))
        })
    });

    let delete_fn: DeleteFn = Arc::new(|db: ADb, id: String| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            runique_release::Entity::delete_by_id(id).exec(&*db).await.map(|_| ())
        })
    });

    let create_fn: CreateFn = Arc::new(|db: ADb, data: StrMap| {
        Box::pin(async move {
            runique_release::admin_from_form(&data, None)
                .insert(&*db).await.map(|_| ())
        })
    });

    let update_fn: UpdateFn = Arc::new(|db: ADb, id: String, data: StrMap| {
        Box::pin(async move {
            let id = id.parse::<i32>().map_err(|_| DbErr::Custom("id invalide".to_string().to_string()))?;
            runique_release::admin_from_form(&data, Some(id))
                .update(&*db).await.map(|_| ())
        })
    });

    let meta = meta.display(DisplayConfig::new().columns_include(vec![("version", "Version"), ("github_url", "GitHub"), ("crates_url", "Crates.io")]).list_filter(vec![("version", "Version", 10u64)]));
    let filter_fn: FilterFn = Arc::new(|db: ADb, pages: std::collections::HashMap<String, u64>| {
        Box::pin(async move {
            use sea_orm::{ConnectionTrait, ExprTrait};
            use sea_orm::sea_query::{Query, Alias, Expr};
            let mut result: std::collections::HashMap<String, (Vec<String>, u64)> = std::collections::HashMap::new();
            let page_size_version = 10u64;
            let cur_page_version = pages.get("version").copied().unwrap_or(0);
            let count_stmt_version = Query::select().expr(Expr::cust("COUNT(DISTINCT version)")).from(Alias::new(runique_release::Entity.table_name())).and_where(Expr::col(Alias::new("version")).is_not_null()).to_owned();
            let count_row_version = match db.query_one(&count_stmt_version).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `runique_release.version` : colonne introuvable en DB — {}", e); None } };
            let total_version = count_row_version.and_then(|r| r.try_get_by_index::<i64>(0).ok()).unwrap_or(0) as u64;
            let stmt_version = Query::select().distinct().expr(Expr::cust("CAST(version AS TEXT)")).from(Alias::new(runique_release::Entity.table_name())).and_where(Expr::col(Alias::new("version")).is_not_null()).limit(page_size_version).offset(cur_page_version * page_size_version).to_owned();
            let rows_version = match db.query_all(&stmt_version).await { Ok(r) => r, Err(e) => { tracing::error!("[runique admin] list_filter `runique_release.version` : colonne introuvable en DB — {}", e); vec![] } };
            let mut vals_version: Vec<String> = rows_version.iter().filter_map(|r| r.try_get_by_index::<String>(0).ok()).collect(); vals_version.sort_by(|a, b| match (a.parse::<i64>(), b.parse::<i64>()) { (Ok(x), Ok(y)) => x.cmp(&y), _ => a.cmp(b) });
            result.insert("version".to_string(), (vals_version, total_version));
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
            .with_filter_fn(filter_fn)
    );

    registry
}

/// Construit le Router axum du prototype admin pour le préfixe donné.
/// À passer à `.with_admin(|a| a.routes(admins::routes("/admin")))` dans main.rs.
pub fn routes(prefix: &str) -> runique::axum::Router {
    let p = prefix.trim_end_matches('/');
    runique::axum::Router::new()
        .route(&format!("{}/{{resource}}/{{action}}", p), get(admin_get).post(admin_post))
        .route(&format!("{}/{{resource}}/{{id}}/{{action}}", p), get(admin_get_id).post(admin_post_id))
}

/// Retourne l'état partagé du prototype admin (pour le dashboard).
pub fn admin_state() -> std::sync::Arc<PrototypeAdminState> {
    let config = Arc::new(AdminConfig::new());
    std::sync::Arc::new(PrototypeAdminState {
        registry: Arc::new(admin_register()),
        config,
    })
}
