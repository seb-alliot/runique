//! The context of each generated page: a template overriding one reads these
//! fields, under these names. They are the contract with a project's own
//! templates: renaming one is a breaking change.
//!
//! Every page also gets the menu (`resources`, `current_resource`,
//! `current_user`, `site_title`, `site_url`, `lang`, `admin_prefix`) and its
//! translated texts (`admin_<page>_<key>`).
use crate::context::template::{AppError, Request};
use crate::errors::error::ErrorContext;
use crate::utils::aliases::{AppResult, StrMap};
use serde::Serialize;

/// The model a page is about.
#[derive(Serialize)]
pub struct ResourceRef {
    /// Its key, the `{resource}` of the URL.
    pub key: &'static str,
    /// Its name in the admin.
    pub title: String,
    /// What its list shows besides the rows.
    pub display: Display,
}

#[derive(Serialize, Default)]
pub struct Display {
    /// The filtered columns, `(column, label)`.
    pub list_filter: Vec<(String, String)>,
}

impl ResourceRef {
    pub(crate) fn new(key: &'static str, title: String) -> Self {
        Self {
            key,
            title,
            display: Display::default(),
        }
    }
}

/// `admin/list.html` and its HTMX part `admin/list_partial.html`.
#[derive(Serialize)]
pub struct ListContext {
    pub resource: ResourceRef,
    /// `{prefix}/{resource}`, for the pages not yet named (create, delete, bulk).
    pub resource_base: String,
    /// `"list"`: the menu entry marked active.
    pub current_page: &'static str,
    /// The rows, each an object of its shown columns and its key; with
    /// `detail_url` when a declared route replaces the detail page.
    pub entries: Vec<serde_json::Value>,
    /// The shown columns, in order, key out.
    pub visible_columns: Vec<&'static str>,
    /// Header of a column, by name; a column without one shows its name.
    pub column_labels: StrMap,
    /// Columns holding HTML, shown as plain text in the list.
    pub rich_fields: Vec<String>,
    /// The declared actions the bar offers: `index` (sent as `a<index>`), `label`.
    pub group_actions: Vec<crate::admin::builder::admin_builder::OfferedAction>,
    /// The list's current query (search, sort, filters, page), so a grouped
    /// action comes back to the same page.
    pub return_qs: String,
    /// Each filter's values, by column (the columns are in
    /// `resource.display.list_filter`).
    pub filter_values: std::collections::HashMap<&'static str, Vec<String>>,
    /// The filters applied, column → value.
    pub active_filters: StrMap,
    /// The applied filters as a query string (`&filter_col=value…`).
    pub filter_qs: String,
    pub search: String,
    pub sort_by: String,
    /// `"asc"` or `"desc"`.
    pub sort_dir: String,
    /// The other direction, for the sorted column's header.
    pub sort_dir_toggle: &'static str,
    /// Rows matching, all pages.
    pub total: u64,
    pub page: u64,
    pub page_count: u64,
    pub has_prev: bool,
    pub has_next: bool,
    pub prev_page: u64,
    pub next_page: u64,
    /// What the row menus and the bars offer; the gate decides again on
    /// the page each one leads to.
    pub can_view: bool,
    pub can_create: bool,
    pub can_update: bool,
    pub can_edit: bool,
    pub can_delete: bool,
    pub can_bulk: bool,
}

/// `admin/detail.html`.
#[derive(Serialize)]
pub struct DetailContext {
    pub resource: ResourceRef,
    /// Same as `resource.key`.
    pub resource_key: &'static str,
    /// `"detail"`.
    pub current_page: &'static str,
    /// The row's key, as in the URL.
    pub object_id: String,
    /// The row: one entry per readable column.
    pub entry: serde_json::Value,
    /// Columns holding HTML, shown sanitized.
    pub rich_fields: Vec<String>,
    /// Whether the edit button is offered.
    pub can_update: bool,
    /// Whether the delete button is offered.
    pub can_delete: bool,
    /// Whether the "reset password" button is offered: an account, and the
    /// right to update it. The gate decides again, rank rule included.
    pub can_reset_password: bool,
    /// On an account the viewer may give a rank to: `(value, label)` of
    /// the ranks below the viewer's. Empty otherwise.
    pub rank_choices: Vec<(&'static str, &'static str)>,
    /// That account's rank now (`lambda`, `admin`, `dirigeant`,
    /// `superuser`), when `rank_choices` isn't empty.
    pub current_rank: Option<&'static str>,
}

/// What the form blocks (`admin/blocks/form/`) read, the same on every form page.
#[derive(Serialize)]
pub struct FormTexts {
    /// Where "cancel" leads: the row on edit, the list on create.
    pub cancel_url: String,
    pub cancel_label: String,
    pub submit_label: String,
    /// Shown when the form has no field.
    pub no_fields_label: String,
}

/// `admin/edit.html`.
#[derive(Serialize)]
pub struct EditContext {
    pub resource: ResourceRef,
    /// Same as `resource.key`.
    pub resource_key: &'static str,
    /// `"edit"`.
    pub current_page: &'static str,
    /// The row's key, as in the URL.
    pub object_id: String,
    /// The form: `html` (its fields, CSRF and form name included), `errors`,
    /// `form_errors`, `fields`.
    pub form_fields: serde_json::Value,
    #[serde(flatten)]
    pub texts: FormTexts,
}

/// `admin/create.html`.
#[derive(Serialize)]
pub struct CreateContext {
    pub resource: ResourceRef,
    /// Same as `resource.key`.
    pub resource_key: &'static str,
    /// `"create"`.
    pub current_page: &'static str,
    /// The form: `html` (its fields, CSRF and form name included), `errors`,
    /// `form_errors`, `fields`.
    pub form_fields: serde_json::Value,
    #[serde(flatten)]
    pub texts: FormTexts,
}

/// `admin/delete.html`.
#[derive(Serialize)]
pub struct DeleteContext {
    pub resource: ResourceRef,
    /// Same as `resource.key`.
    pub resource_key: &'static str,
    /// `"delete"`.
    pub current_page: &'static str,
    /// The row's key, as in the URL.
    pub object_id: String,
    /// The confirmation form: `html` (its CSRF token and name), `errors`,
    /// `form_errors`. It has no field.
    pub form_fields: serde_json::Value,
    /// Where "cancel" leads: the row.
    pub cancel_url: String,
}

/// One history entry as the pages show it.
#[derive(Serialize)]
pub struct HistoryRow {
    pub id: i64,
    /// `YYYY-MM-DD HH:MM:SS`, UTC.
    pub created_at: String,
    pub username: String,
    pub resource_key: String,
    /// The model's name in the admin.
    pub resource_title: String,
    /// The row's key when the action happened.
    pub object_pk: String,
    /// The row's label then, when its column is readable; else none.
    pub object_label: Option<String>,
    /// `create`, `update` or `delete`.
    pub action: String,
    /// Whether the entry kept changes the account may read.
    pub has_changes: bool,
}

/// One changed column of an entry.
#[derive(Serialize)]
pub struct HistoryChange {
    pub column: String,
    pub old: String,
    pub new: String,
}

/// A model the history can be filtered on.
#[derive(Serialize)]
pub struct HistoryResource {
    pub key: &'static str,
    pub title: String,
}

/// `admin/history.html`.
#[derive(Serialize)]
pub struct HistoryContext {
    /// `"history"`.
    pub current_page: &'static str,
    /// The newest first.
    pub entries: Vec<HistoryRow>,
    /// The models whose history this account reads, for the filter. Not
    /// `resources`: that key is the menu's.
    pub history_resources: Vec<HistoryResource>,
    pub filter_resource: String,
    pub filter_object: String,
    pub filter_action: String,
    /// An account's username.
    pub filter_user: String,
    pub page: u64,
    pub total_pages: u64,
}

/// `admin/history_diff.html`.
#[derive(Serialize)]
pub struct HistoryEntryContext {
    /// `"history"`.
    pub current_page: &'static str,
    pub entry: HistoryRow,
    /// Its readable changed columns.
    pub changes: Vec<HistoryChange>,
}

/// Adds `page`'s fields to the request's template context.
pub(crate) fn put<T: Serialize>(req: &mut Request, page: &T) -> AppResult<()> {
    let context = tera::Context::from_serialize(page).map_err(|e| {
        Box::new(AppError::new(ErrorContext::generic(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            &format!("admin page context: {e}"),
        )))
    })?;
    req.context.extend(context);
    Ok(())
}
