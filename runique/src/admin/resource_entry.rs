use std::sync::Arc;

use axum::http::Method;
use futures_util::future::BoxFuture;
use sea_orm::DbErr;
use serde_json::Value;

use crate::admin::dyn_form::DynForm;
use crate::admin::resource::AdminResource;
use crate::utils::aliases::{ADb, ATera, StrMap};

/// Closure construisant un form typé depuis des données brutes.
pub type FormBuilder = Arc<
    dyn Fn(StrMap, ATera, String, Method) -> BoxFuture<'static, Box<dyn DynForm>> + Send + Sync,
>;

/// Closure retournant toutes les entrées d'une ressource sous forme de `Vec<Value>`.
pub type ListFn = Arc<dyn Fn(ADb) -> BoxFuture<'static, Result<Vec<Value>, DbErr>> + Send + Sync>;

/// Closure retournant une entrée par son id sous forme de `Value`.
pub type GetFn =
    Arc<dyn Fn(ADb, i32) -> BoxFuture<'static, Result<Option<Value>, DbErr>> + Send + Sync>;

/// Closure supprimant une entrée par son id.
pub type DeleteFn = Arc<dyn Fn(ADb, i32) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure mettant à jour une entrée par son id depuis les données du formulaire validé.
pub type UpdateFn =
    Arc<dyn Fn(ADb, i32, StrMap) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure créant une nouvelle entrée depuis les données du formulaire validé.
pub type CreateFn = Arc<dyn Fn(ADb, StrMap) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure retournant le nombre total d'entrées d'une ressource.
pub type CountFn = Arc<dyn Fn(ADb) -> BoxFuture<'static, Result<u64, DbErr>> + Send + Sync>;

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
}

/// Opérations CRUD disponibles sur une ressource admin
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum CrudOperation {
    List,
    View,
    Create,
    Edit,
    Delete,
}

/// Filtre les colonnes affichées dans la vue liste
#[derive(Debug, Clone, Default, serde::Serialize)]
pub enum ColumnFilter {
    /// Affiche toutes les colonnes du Model (défaut)
    #[default]
    All,

    /// Affiche uniquement les colonnes spécifiées
    Include(Vec<String>),

    /// Affiche toutes les colonnes sauf celles spécifiées
    Exclude(Vec<String>),
}

/// Configuration de l'affichage d'une ressource dans l'interface admin
#[derive(Debug, Clone, serde::Serialize)]
pub struct DisplayConfig {
    /// Icône affichée dans la navigation (nom d'icône, ex: "user", "file")
    pub icon: Option<String>,

    /// Colonnes à afficher dans la vue liste
    pub columns: ColumnFilter,

    /// Nombre d'entrées par page
    pub pagination: usize,
}

impl DisplayConfig {
    pub fn new() -> Self {
        Self {
            icon: None,
            columns: ColumnFilter::All,
            pagination: 25,
        }
    }

    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = Some(icon.to_string());
        self
    }

    pub fn pagination(mut self, per_page: usize) -> Self {
        self.pagination = per_page;
        self
    }

    pub fn columns_include(mut self, cols: Vec<&str>) -> Self {
        self.columns = ColumnFilter::Include(cols.iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn columns_exclude(mut self, cols: Vec<&str>) -> Self {
        self.columns = ColumnFilter::Exclude(cols.iter().map(|s| s.to_string()).collect());
        self
    }
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self::new()
    }
}
