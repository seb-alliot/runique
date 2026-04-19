//! Admin resource entry: CRUD callbacks, metadata, and display configuration.
use std::sync::Arc;

use axum::http::Method;
use futures_util::future::BoxFuture;
use sea_orm::DbErr;
use serde_json::Value;

pub use crate::admin::{
    helper::dyn_form::DynForm,
    resource::{AdminResource, ColumnFilter, CrudOperation, DisplayConfig},
};
use crate::utils::aliases::{ADb, ATera, StrMap};

/// Sort direction for the admin list view.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize)]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

impl SortDir {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            SortDir::Asc => "asc",
            SortDir::Desc => "desc",
        }
    }
    #[must_use]
    pub fn toggle(&self) -> &'static str {
        match self {
            SortDir::Asc => "desc",
            SortDir::Desc => "asc",
        }
    }
}

/// Parameters passed to `ListFn`: pagination, sorting, search, column filters.
#[derive(Debug, Clone)]
pub struct ListParams {
    pub offset: u64,
    pub limit: u64,
    pub sort_by: Option<String>,
    pub sort_dir: SortDir,
    pub search: Option<String>,
    /// Exact filters by column: [(`col_sql`, `value`)]
    pub column_filters: Vec<(String, String)>,
}

/// Closure building a typed form from raw data.
pub type FormBuilder = Arc<
    dyn Fn(ADb, Vec<String>, StrMap, ATera, String, Method) -> BoxFuture<'static, Box<dyn DynForm>>
        + Send
        + Sync,
>;

/// Closure returning a page of resource entries.
pub type ListFn =
    Arc<dyn Fn(ADb, ListParams) -> BoxFuture<'static, Result<Vec<Value>, DbErr>> + Send + Sync>;

/// Closure returning an entry by its ID as a `Value`.
pub type GetFn =
    Arc<dyn Fn(ADb, String) -> BoxFuture<'static, Result<Option<Value>, DbErr>> + Send + Sync>;

/// Closure deleting an entry by its ID.
pub type DeleteFn = Arc<dyn Fn(ADb, String) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure updating an entry by its ID from validated form data.
pub type UpdateFn =
    Arc<dyn Fn(ADb, String, StrMap) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure creating a new entry from validated form data.
pub type CreateFn = Arc<dyn Fn(ADb, StrMap) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure returning the total number of entries (with optional search term).
pub type CountFn =
    Arc<dyn Fn(ADb, Option<String>) -> BoxFuture<'static, Result<u64, DbErr>> + Send + Sync>;

/// Closure returning distinct values for each column configured in `list_filter`.
/// Parameter: current page per column (0-based).
/// Returns: `HashMap`<`col_sql`, (`page_values`, `total_distinct`)>
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

/// One field available for group (bulk) update in the list view.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GroupAction {
    pub field: String,
    pub label: String,
    pub choices: Vec<(String, String)>,
}

impl GroupAction {
    /// Boolean field: 3-state select (empty = no change, true/false).
    pub fn bool(field: &str, label: &str) -> Self {
        Self {
            field: field.to_string(),
            label: label.to_string(),
            choices: vec![
                ("true".to_string(), "Oui".to_string()),
                ("false".to_string(), "Non".to_string()),
            ],
        }
    }
}

/// Admin registry entry: metadata + CRUD closures.
pub struct ResourceEntry {
    pub meta: AdminResource,
    pub form_builder: FormBuilder,
    pub edit_form_builder: Option<FormBuilder>,
    pub list_fn: Option<ListFn>,
    pub get_fn: Option<GetFn>,
    pub delete_fn: Option<DeleteFn>,
    pub update_fn: Option<UpdateFn>,
    pub partial_update_fn: Option<UpdateFn>,
    pub create_fn: Option<CreateFn>,
    pub count_fn: Option<CountFn>,
    pub filter_fn: Option<FilterFn>,
    pub group_actions: Vec<GroupAction>,
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
            partial_update_fn: None,
            create_fn: None,
            count_fn: None,
            filter_fn: None,
            group_actions: Vec::new(),
        }
    }
    #[must_use]
    pub fn with_edit_form_builder(mut self, f: FormBuilder) -> Self {
        self.edit_form_builder = Some(f);
        self
    }
    #[must_use]
    pub fn with_list_fn(mut self, f: ListFn) -> Self {
        self.list_fn = Some(f);
        self
    }
    #[must_use]
    pub fn with_get_fn(mut self, f: GetFn) -> Self {
        self.get_fn = Some(f);
        self
    }
    #[must_use]
    pub fn with_delete_fn(mut self, f: DeleteFn) -> Self {
        self.delete_fn = Some(f);
        self
    }
    #[must_use]
    pub fn with_update_fn(mut self, f: UpdateFn) -> Self {
        self.update_fn = Some(f);
        self
    }
    #[must_use]
    pub fn with_partial_update_fn(mut self, f: UpdateFn) -> Self {
        self.partial_update_fn = Some(f);
        self
    }
    #[must_use]
    pub fn with_create_fn(mut self, f: CreateFn) -> Self {
        self.create_fn = Some(f);
        self
    }
    #[must_use]
    pub fn with_count_fn(mut self, f: CountFn) -> Self {
        self.count_fn = Some(f);
        self
    }
    #[must_use]
    pub fn with_filter_fn(mut self, f: FilterFn) -> Self {
        self.filter_fn = Some(f);
        self
    }
    #[must_use]
    pub fn with_group_actions(mut self, actions: Vec<GroupAction>) -> Self {
        self.group_actions = actions;
        self
    }
}
