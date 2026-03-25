use std::sync::Arc;

use axum::http::Method;
use futures_util::future::BoxFuture;
use sea_orm::DbErr;
use serde_json::Value;

pub use crate::admin::{
    dyn_form::DynForm,
    resource::{AdminResource, ColumnFilter, CrudOperation, DisplayConfig},
};
use crate::utils::aliases::{ADb, ATera, StrMap};

/// Direction de tri pour la vue liste admin.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

impl SortDir {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortDir::Asc => "asc",
            SortDir::Desc => "desc",
        }
    }

    pub fn toggle(&self) -> &'static str {
        match self {
            SortDir::Asc => "desc",
            SortDir::Desc => "asc",
        }
    }
}

/// Paramètres passés à `ListFn` : pagination, tri, recherche, filtres colonne.
#[derive(Debug, Clone)]
pub struct ListParams {
    pub offset: u64,
    pub limit: u64,
    pub sort_by: Option<String>,
    pub sort_dir: SortDir,
    pub search: Option<String>,
    /// Filtres exacts par colonne : [(col_sql, valeur)]
    pub column_filters: Vec<(String, String)>,
}

/// Closure construisant un form typé depuis des données brutes.
pub type FormBuilder = Arc<
    dyn Fn(StrMap, ATera, String, Method) -> BoxFuture<'static, Box<dyn DynForm>> + Send + Sync,
>;

/// Closure retournant une page d'entrées d'une ressource.
pub type ListFn =
    Arc<dyn Fn(ADb, ListParams) -> BoxFuture<'static, Result<Vec<Value>, DbErr>> + Send + Sync>;

/// Closure retournant une entrée par son id sous forme de `Value`.
pub type GetFn =
    Arc<dyn Fn(ADb, String) -> BoxFuture<'static, Result<Option<Value>, DbErr>> + Send + Sync>;

/// Closure supprimant une entrée par son id.
pub type DeleteFn = Arc<dyn Fn(ADb, String) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure mettant à jour une entrée par son id depuis les données du formulaire validé.
pub type UpdateFn =
    Arc<dyn Fn(ADb, String, StrMap) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure créant une nouvelle entrée depuis les données du formulaire validé.
pub type CreateFn = Arc<dyn Fn(ADb, StrMap) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure retournant le nombre total d'entrées (avec terme de recherche optionnel).
pub type CountFn =
    Arc<dyn Fn(ADb, Option<String>) -> BoxFuture<'static, Result<u64, DbErr>> + Send + Sync>;

/// Closure retournant les valeurs distinctes de chaque colonne configurée dans list_filter.
/// Paramètre : page courante par colonne (0-based).
/// Retourne : HashMap<col_sql, (valeurs_de_la_page, total_distinct)>
pub type FilterFn = Arc<
    dyn Fn(
            ADb,
            std::collections::HashMap<String, u64>,
        ) -> BoxFuture<
            'static,
            Result<std::collections::HashMap<String, (Vec<String>, u64)>, DbErr>,
        > + Send
        + Sync,
>;

/// Entrée du registre admin : métadonnées + closures CRUD.
pub struct ResourceEntry {
    pub meta: AdminResource,
    pub form_builder: FormBuilder,
    pub edit_form_builder: Option<FormBuilder>,
    pub list_fn: Option<ListFn>,
    pub get_fn: Option<GetFn>,
    pub delete_fn: Option<DeleteFn>,
    pub update_fn: Option<UpdateFn>,
    pub create_fn: Option<CreateFn>,
    pub count_fn: Option<CountFn>,
    pub filter_fn: Option<FilterFn>,
}

impl ResourceEntry {
    pub fn new(meta: AdminResource, form_builder: FormBuilder) -> Self {
        Self {
            meta,
            form_builder,
            edit_form_builder: None,
            list_fn: None,
            get_fn: None,
            delete_fn: None,
            update_fn: None,
            create_fn: None,
            count_fn: None,
            filter_fn: None,
        }
    }

    pub fn with_edit_form_builder(mut self, f: FormBuilder) -> Self {
        self.edit_form_builder = Some(f);
        self
    }

    pub fn with_list_fn(mut self, f: ListFn) -> Self {
        self.list_fn = Some(f);
        self
    }

    pub fn with_get_fn(mut self, f: GetFn) -> Self {
        self.get_fn = Some(f);
        self
    }

    pub fn with_delete_fn(mut self, f: DeleteFn) -> Self {
        self.delete_fn = Some(f);
        self
    }

    pub fn with_update_fn(mut self, f: UpdateFn) -> Self {
        self.update_fn = Some(f);
        self
    }

    pub fn with_create_fn(mut self, f: CreateFn) -> Self {
        self.create_fn = Some(f);
        self
    }

    pub fn with_count_fn(mut self, f: CountFn) -> Self {
        self.count_fn = Some(f);
        self
    }

    pub fn with_filter_fn(mut self, f: FilterFn) -> Self {
        self.filter_fn = Some(f);
        self
    }
}
