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
    /// Returns the query-string representation of this direction (`"asc"` or `"desc"`).
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            SortDir::Asc => "asc",
            SortDir::Desc => "desc",
        }
    }
    /// Returns the opposite direction as a string, used to flip sort order
    /// when the same column header is clicked again.
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
    /// Trusted parent scope `Some((fk_col, parent_id))` when this resource is
    /// listed as a scoped child. Applied unconditionally (framework-injected,
    /// never from the query string) — bypasses the sidebar-filter allowlist.
    pub scope: Option<(String, String)>,
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

/// Rewrites a serialized row's enum columns to their display label. Model-provided
/// (only the `model!` macro knows the model's enums), applied at the display layer
/// so the edit form keeps the raw db value. A plain `fn` pointer (`Send + Sync`).
pub type EnumLabelFn = fn(&mut Value);

/// Closure deleting an entry by its ID.
pub type DeleteFn = Arc<dyn Fn(ADb, String) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure updating an entry by its ID from validated form data.
pub type UpdateFn =
    Arc<dyn Fn(ADb, String, StrMap) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure creating a new entry from validated form data.
pub type CreateFn = Arc<dyn Fn(ADb, StrMap) -> BoxFuture<'static, Result<(), DbErr>> + Send + Sync>;

/// Closure returning the total number of entries.
///
/// Receives the optional search term **and** the trusted parent scope
/// (`Some((fk_col, parent_id))`), so a scoped child list
/// (`WHERE {fk_col} = {parent_id}`) paginates on the scoped total rather than
/// the whole table. The scope is framework-injected (never from the query
/// string), so it bypasses the sidebar-filter allowlist.
pub type CountFn = Arc<
    dyn Fn(ADb, Option<String>, Option<(String, String)>) -> BoxFuture<'static, Result<u64, DbErr>>
        + Send
        + Sync,
>;

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

/// Options for a single M2M field, passed to the create/edit template context.
#[derive(Debug, Clone, serde::Serialize)]
pub struct M2mFieldOptions {
    /// Form field name (e.g., "allergenes")
    pub field_name: String,
    /// Human-readable label (e.g., "Allergènes")
    pub label: String,
    /// All available choices: (id as String, display label)
    pub choices: Vec<(String, String)>,
    /// Currently selected IDs (empty for create, pre-filled for edit)
    pub selected: Vec<String>,
}

/// Loads M2M field options for the create/edit form.
/// `object_id`: None for create, Some(id) for edit (to pre-select existing relations).
pub type M2mLoaderFn =
    Arc<dyn Fn(ADb, Option<String>) -> BoxFuture<'static, Vec<M2mFieldOptions>> + Send + Sync>;

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

    /// Single-value action: submits exactly one fixed value for the field.
    pub fn val(field: &str, label: &str, value: &str) -> Self {
        Self {
            field: field.to_string(),
            label: label.to_string(),
            choices: vec![(value.to_string(), label.to_string())],
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
    pub m2m_loader: Option<M2mLoaderFn>,
    pub unique_fields: &'static [&'static str],
    /// Field name used to verify record ownership when `can_update_own`/`can_delete_own` is set.
    /// If None, "own" permissions are blocked (safe default until the field is declared).
    pub own_field: Option<&'static str>,
    /// Model-provided resolver turning enum db values into display labels (display views).
    pub enum_label_fn: Option<EnumLabelFn>,
}

impl ResourceEntry {
    /// Creates a resource entry with only metadata and a form builder set;
    /// every CRUD closure defaults to `None` until wired up via the `with_*` builders.
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
            m2m_loader: None,
            unique_fields: &[],
            own_field: None,
            enum_label_fn: None,
        }
    }
    /// Registers the enum-label resolver used to turn raw enum db values into
    /// display labels in list/detail/delete views. Used by generated admin
    /// resource code; not typically called directly.
    #[must_use]
    pub fn with_enum_label_fn(mut self, f: EnumLabelFn) -> Self {
        self.enum_label_fn = Some(f);
        self
    }
    /// Registers the closure that loads many-to-many field options for the
    /// create/edit form.
    #[must_use]
    pub fn with_m2m_loader(mut self, f: M2mLoaderFn) -> Self {
        self.m2m_loader = Some(f);
        self
    }
    /// Registers a dedicated form builder for the edit view, used instead of
    /// `form_builder` when the edit form differs from the create form.
    #[must_use]
    pub fn with_edit_form_builder(mut self, f: FormBuilder) -> Self {
        self.edit_form_builder = Some(f);
        self
    }
    /// Registers the closure that returns a page of resource entries for the list view.
    #[must_use]
    pub fn with_list_fn(mut self, f: ListFn) -> Self {
        self.list_fn = Some(f);
        self
    }
    /// Registers the closure that fetches a single entry by its ID.
    #[must_use]
    pub fn with_get_fn(mut self, f: GetFn) -> Self {
        self.get_fn = Some(f);
        self
    }
    /// Registers the closure that deletes an entry by its ID.
    #[must_use]
    pub fn with_delete_fn(mut self, f: DeleteFn) -> Self {
        self.delete_fn = Some(f);
        self
    }
    /// Registers the closure that fully updates an entry from validated form data.
    #[must_use]
    pub fn with_update_fn(mut self, f: UpdateFn) -> Self {
        self.update_fn = Some(f);
        self
    }
    /// Registers the closure used for partial updates (e.g. group/bulk field updates).
    #[must_use]
    pub fn with_partial_update_fn(mut self, f: UpdateFn) -> Self {
        self.partial_update_fn = Some(f);
        self
    }
    /// Registers the closure that creates a new entry from validated form data.
    #[must_use]
    pub fn with_create_fn(mut self, f: CreateFn) -> Self {
        self.create_fn = Some(f);
        self
    }
    /// Registers the closure that returns the total row count, used for pagination.
    #[must_use]
    pub fn with_count_fn(mut self, f: CountFn) -> Self {
        self.count_fn = Some(f);
        self
    }
    /// Registers the closure that returns the distinct values available for
    /// each `list_filter` sidebar column.
    #[must_use]
    pub fn with_filter_fn(mut self, f: FilterFn) -> Self {
        self.filter_fn = Some(f);
        self
    }
    /// Declares which field names carry a unique DB constraint on the underlying
    /// model. Used to exclude them from bulk edit, since applying the same value
    /// to multiple rows would violate the constraint.
    #[must_use]
    pub fn with_unique_fields(mut self, fields: &'static [&'static str]) -> Self {
        self.unique_fields = fields;
        self
    }
    /// Declares the field used to verify record ownership when
    /// `can_update_own`/`can_delete_own` permissions are set.
    #[must_use]
    pub fn with_own_field(mut self, field: &'static str) -> Self {
        self.own_field = Some(field);
        self
    }
    /// Registers the bulk (group) actions available in the list view, merging
    /// choices together for actions declared multiple times on the same field.
    #[must_use]
    pub fn with_group_actions(mut self, actions: Vec<GroupAction>) -> Self {
        let mut merged: Vec<GroupAction> = Vec::new();
        for action in actions {
            if let Some(existing) = merged.iter_mut().find(|a| a.field == action.field) {
                existing.choices.extend(action.choices);
            } else {
                merged.push(action);
            }
        }
        self.group_actions = merged;
        self
    }
}
