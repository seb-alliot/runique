//! The context of each generated page: a template overriding one reads these
//! fields, under these names. They are the contract with a project's own
//! templates: renaming one is a breaking change.
//!
//! Every page also gets the menu (`nav`, `resources`, `current_resource`,
//! `current_user`, `site_title`, `lang`) and its translated texts
//! (`admin_<page>_<key>`).
//!
//! Every link comes built: a page never assembles a URL from a route name
//! or the admin's prefix. A link the account may not follow is `None`, and
//! its button isn't shown; the gate decides again on the page it leads to.
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

/// The menu's links, on every page (`nav`).
#[derive(Serialize)]
pub struct NavLinks {
    pub dashboard: String,
    /// The history, when the account reads one.
    pub history: Option<String>,
    /// Where the sign-out form posts.
    pub logout: String,
    /// "Back to the site".
    pub site: String,
    /// The dashboard's switch back to the configured template.
    pub toggle_template: String,
}

/// A list's own links.
#[derive(Serialize)]
pub struct ResourceLinks {
    pub list: String,
    pub create: Option<String>,
    /// Where the grouped actions' bar posts, when it offers any.
    pub bulk: Option<String>,
    /// On the groups' list, the shortcut to their rights, when the account
    /// may set one.
    pub rights: Option<String>,
}

/// A row's links: its list menu, its detail page's buttons.
#[derive(Serialize)]
pub struct RowLinks {
    /// The row's page: the route the developer declared to replace it, if any.
    pub detail: String,
    pub edit: Option<String>,
    pub delete: Option<String>,
}

/// One row of a list.
#[derive(Serialize)]
pub struct ListEntry {
    /// Its key, as in the URL.
    pub id: String,
    /// Its shown columns and its key, by column name.
    pub values: serde_json::Value,
    pub links: RowLinks,
    /// On a group taken back after a departure: who took it.
    pub taken_by: Option<String>,
}

/// `admin/list.html` and its HTMX part `admin/list_partial.html`.
#[derive(Serialize)]
pub struct ListContext {
    pub resource: ResourceRef,
    pub resource_links: ResourceLinks,
    /// `"list"`: the menu entry marked active.
    pub current_page: &'static str,
    pub entries: Vec<ListEntry>,
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
    /// Whether the grouped actions' bar offers delete.
    pub bulk_delete: bool,
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
    pub list_url: String,
    pub links: RowLinks,
    /// Where the "reset password" form posts: on an account, with the right
    /// to update it. The gate decides again, rank rule included.
    pub reset_password_url: Option<String>,
    /// On an account the viewer may give a rank to: `(value, label)` of
    /// the ranks below the viewer's. Empty otherwise.
    pub rank_choices: Vec<(&'static str, String)>,
    /// That account's rank now (`lambda`, `admin`, `dirigeant`,
    /// `superuser`), when `rank_choices` isn't empty.
    pub current_rank: Option<&'static str>,
    /// Where the rank form posts, when `rank_choices` isn't empty.
    pub rank_url: Option<String>,
    /// The admin's rights grid, when the viewer may set it.
    pub rights_url: Option<String>,
    /// On an account, the groups the viewer manages, ticked when it is a
    /// member: `(form name, group name, ticked)`. Empty otherwise.
    pub memberships: Vec<(String, String, bool)>,
    /// Where the groups' form posts, when `memberships` isn't empty.
    pub memberships_url: Option<String>,
    /// On a group, for a dirigeant: the accounts it may be handed to,
    /// `(id, username)`, and the transfer page their selector opens.
    pub transfer_receivers: Vec<(String, String)>,
    pub transfer_url: Option<String>,
    /// On a group taken back after a departure: who took it. Shown as
    /// "a responsable is to be named".
    pub taken_by: Option<String>,
}

/// One of the signed-in account's groups, on the dashboard.
#[derive(Serialize)]
pub struct MyGroup {
    pub name: String,
    pub staff: bool,
    /// What it lets the account do, table by table; empty when nothing.
    pub rights: Vec<MyGroupRight>,
}

/// A table a group opens, and its rights, comma-separated.
#[derive(Serialize)]
pub struct MyGroupRight {
    pub table: String,
    pub rights: String,
}

/// `admin/transfer.html`: what handing a group over does, and its form.
#[derive(Serialize)]
pub struct TransferContext {
    pub resource: ResourceRef,
    /// `"transfer"`.
    pub current_page: &'static str,
    pub list_url: String,
    pub detail_url: String,
    /// Where the confirmation posts.
    pub form_action: String,
    /// The group's name.
    pub group: String,
    /// Its responsable now, and the receiver.
    pub from: String,
    pub to: String,
    pub to_id: String,
    /// The rights at stake: kept, the receiver gets them; not kept, the
    /// members lose them. Empty when the receiver covers the whole group.
    pub lines: Vec<TransferLine>,
    /// The same lines as the gate compares them, sent back with the
    /// confirmation.
    pub shown: String,
}

/// One right at stake in a transfer.
#[derive(Serialize)]
pub struct TransferLine {
    /// Its box.
    pub name: String,
    pub table: String,
    pub right: String,
    /// Ticked at first.
    pub kept: bool,
    /// False for a full right on the groups: lost, never given to an admin.
    pub keepable: bool,
}

/// What the form blocks (`admin/blocks/form/`) read, the same on every form page.
#[derive(Serialize)]
pub struct FormTexts {
    /// Where the form posts.
    pub form_action: String,
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
    pub list_url: String,
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
    pub list_url: String,
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
    pub list_url: String,
    /// Where the confirmation posts.
    pub form_action: String,
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
    pub links: HistoryLinks,
}

/// An entry's links.
#[derive(Serialize)]
pub struct HistoryLinks {
    /// The history of its account.
    pub user: String,
    /// The history of its row.
    pub object: String,
    /// Its changes, when it kept some the account may read.
    pub entry: Option<String>,
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
    /// The whole history: where the filters post, and their reset.
    pub history_url: String,
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
    pub history_url: String,
    pub entry: HistoryRow,
    /// Its readable changed columns.
    pub changes: Vec<HistoryChange>,
}

/// One right of the grid: its form name, and whether it is ticked.
#[derive(Serialize)]
pub struct GridBox {
    /// The checkbox's `name`, what the page sends back when ticked.
    pub name: String,
    pub ticked: bool,
}

/// One column of a table in the grid.
#[derive(Serialize)]
pub struct GridColumn {
    pub column: &'static str,
    /// `None`: a right the viewer may not give, left out of the grid.
    pub read: Option<GridBox>,
    pub create: Option<GridBox>,
    pub update: Option<GridBox>,
}

/// One table of the grid: its table rights, then its columns.
#[derive(Serialize)]
pub struct GridTable {
    pub key: &'static str,
    pub title: String,
    /// `(label, box)` for create, read, read_own, update, update_own,
    /// delete, delete_own.
    pub table: Vec<(String, GridBox)>,
    pub columns: Vec<GridColumn>,
}

/// `admin/rights.html`: an admin's own rights, set by a dirigeant.
#[derive(Serialize)]
pub struct RightsContext {
    pub resource: ResourceRef,
    /// Same as `resource.key`.
    pub resource_key: &'static str,
    /// `"rights"`.
    pub current_page: &'static str,
    pub list_url: String,
    /// Where the grid posts.
    pub form_action: String,
    /// The admin's key, as in the URL.
    pub object_id: String,
    pub account_name: String,
    /// Every model of the admin, accounts first.
    pub tables: Vec<GridTable>,
    /// A group's grid (its members' rights) rather than an admin's.
    pub is_group: bool,
    /// On a group's grid, the groups the viewer may set, to pick another:
    /// `(id, name)`, by name.
    pub group_choices: Vec<(String, String)>,
    /// Where the selector sends the group picked (GET, a redirect).
    pub group_select_url: Option<String>,
    /// The form's CSRF token and name: `html`.
    pub form_fields: serde_json::Value,
    /// The account's page.
    pub cancel_url: String,
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
