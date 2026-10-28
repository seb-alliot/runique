//! Tera context key constants for admin templates (list, form, etc.).

/// Tera context keys injected into admin templates.
/// Use these constants to override a template without risking typos.
pub mod admin_context {
    /// Shared keys — injected into all admin views
    pub mod common {
        /// Current UI language code (e.g. `"en"`, `"fr"`), injected on every admin page.
        pub const LANG: &str = "lang";
        /// Site title shown in the admin header/tab, taken from `AdminConfig`.
        pub const SITE_TITLE: &str = "site_title";
        /// Base URL of the public site, used to build the "back to site" link.
        pub const SITE_URL: &str = "site_url";
        /// Metadata (`ResourceEntry::meta`) of the resource currently being viewed.
        pub const RESOURCE: &str = "resource";
        /// Key of the resource currently being viewed (e.g. `"menus"`), used to build URLs.
        pub const RESOURCE_KEY: &str = "resource_key";
        /// Alias of [`RESOURCE_KEY`] kept for templates that highlight the active
        /// entry in the resource navigation.
        pub const CURRENT_RESOURCE: &str = "current_resource";
        /// List of resources visible to the current user, shown in the admin dashboard/nav.
        pub const RESOURCES: &str = "resources";
        /// The object being shown, as a `serde_json::Value` row — used by the detail template.
        pub const ENTRY: &str = "entry";
        /// Primary key of the object being viewed/edited/deleted.
        pub const OBJECT_ID: &str = "object_id";
        /// Rendered form fields (`Forms::get_form()`), injected into create/edit/bulk_edit templates.
        pub const FORM_FIELDS: &str = "form_fields";
        /// `true` on the edit form, `false` on create — lets the shared template adjust
        /// labels/actions (e.g. "Save" vs "Create").
        pub const IS_EDIT: &str = "is_edit";
    }

    /// Form field names of the built-in user admin form.
    pub mod user {
        /// The `is_active` flag.
        pub const IS_ACTIVE: &str = "is_active";
    }

    /// `list` template — resource list view
    pub mod list {
        pub use super::common::LANG;
        /// Current page of rows, as `serde_json::Value` objects, after formatting
        /// (e.g. datetimes) and column filtering.
        pub const ENTRIES: &str = "entries";
        /// Total number of rows matching the current search/filters (all pages).
        pub const TOTAL: &str = "total";
        /// Current page number (1-based).
        pub const PAGE: &str = "page";
        /// Total number of pages for the current search/filters.
        pub const PAGE_COUNT: &str = "page_count";
        /// Whether a previous page exists.
        pub const HAS_PREV: &str = "has_prev";
        /// Whether a next page exists.
        pub const HAS_NEXT: &str = "has_next";
        /// Page number to link to for "previous" (clamped, may be unused if `HAS_PREV` is false).
        pub const PREV_PAGE: &str = "prev_page";
        /// Page number to link to for "next" (clamped, may be unused if `HAS_NEXT` is false).
        pub const NEXT_PAGE: &str = "next_page";
        /// Columns to render, resolved from the resource's `ColumnFilter` (see `resolve_columns`).
        pub const VISIBLE_COLUMNS: &str = "visible_columns";
        /// Column name → label; a column without one shows its name.
        pub const COLUMN_LABELS: &str = "column_labels";
        /// Column currently used to sort the list.
        pub const SORT_BY: &str = "sort_by";
        /// Current sort direction (`"asc"`/`"desc"`).
        pub const SORT_DIR: &str = "sort_dir";
        /// Opposite of `SORT_DIR` — used to build the "click to reverse sort" link.
        pub const SORT_DIR_TOGGLE: &str = "sort_dir_toggle";
        /// Current search query string, if any.
        pub const SEARCH: &str = "search";
        /// Currently selected value for each column filter (`list_filter` in `admin!{}`).
        pub const FILTER_VALUES: &str = "filter_values";
        /// Filters that are actually applied (non-empty), used to render active-filter chips.
        pub const ACTIVE_FILTERS: &str = "active_filters";
        /// Query string fragment carrying the active filters, appended to pagination/sort links.
        pub const FILTER_QS: &str = "filter_qs";
        /// Per-column filter pagination metadata (current page, total pages, prev/next query
        /// strings) for filter dropdowns that page their own options.
        pub const FILTER_META: &str = "filter_meta";
        /// Full query string (sort + search + active filters) — pass to edit/delete links so the list
        /// state is restored after returning. Used by `kebab.html` to build the `?return_qs=` param.
        pub const RETURN_QS: &str = "return_qs";
        /// Group actions declared in `admin!{}` — `Vec<GroupAction>` iterated as `ga` in the template.
        pub const GROUP_ACTIONS: &str = "group_actions";
    }

    /// `create` template — creation form
    pub mod create {
        pub use super::common::{FORM_FIELDS, IS_EDIT, LANG};
    }

    /// `edit` template — edition form
    pub mod edit {
        pub use super::common::{FORM_FIELDS, IS_EDIT, LANG, OBJECT_ID};
        /// Injected on GET from the stored object, and on POST re-injected on validation failure.
        /// Used by the optimistic locking check (`__original_updated_at` hidden field).
        pub const ORIG_UPDATED_AT: &str = "orig_updated_at";
        /// Query string from the originating list (sort + search + filters) — used to redirect
        /// back to the exact list page after a successful save.
        pub const RETURN_QS: &str = "return_qs";
    }

    /// `detail` template — object detail view
    pub mod detail {
        pub use super::common::{ENTRY, OBJECT_ID};
        /// Whether the object is an account (accounts table): the password
        /// reset button shows only then, on the same rule as the server.
        pub const CAN_RESET_PASSWORD: &str = "can_reset_password";
    }

    /// `delete` template — deletion confirmation
    pub mod delete {
        pub use super::common::{ENTRY, OBJECT_ID};
    }

    /// `bulk_edit` template — multi-row batch edit form
    pub mod bulk_edit {
        pub use super::common::{FORM_FIELDS, LANG};
        /// Number of selected rows being edited.
        pub const BULK_COUNT: &str = "bulk_count";
        /// Comma-separated IDs of the selected rows — value of the hidden `ids` field.
        pub const BULK_IDS: &str = "bulk_ids";
    }
}

/// Resource keys of the framework's tables: the key of their `ModelMeta` and
/// of their rows in `eihwaz_groupes_droits`.
pub mod resource_key {
    /// Accounts (`eihwaz_users`).
    pub const USERS: &str = "users";
    /// Groups (`eihwaz_groupes`).
    pub const GROUPES: &str = "groupes";
    /// Rights (`eihwaz_groupes_droits`).
    pub const DROITS: &str = "droits";
    /// Memberships (`eihwaz_users_groupes`).
    pub const USERS_GROUPES: &str = "users_groupes";

    /// A group's row never targets one of these: the database refuses it
    /// (CHECK). Changing this list on a live base needs a migration.
    pub const FRAMEWORK_KEYS: &[&str] = &[USERS, GROUPES, DROITS, USERS_GROUPES];
}

/// i18n message keys for the built-in admin templates, grouped by section
/// (`admin.<section>.*`). [`insert_admin_messages`](crate::admin::trad::insert_admin_messages)
/// filters this list by the `admin.<section>.` prefix and injects each match into the
/// Tera context under its dotted key rewritten to `_` (e.g. `admin.login.title` →
/// `admin_login_title`).
pub const ADMIN_MESSAGE_KEYS: &[&str] = &[
    // login
    "admin.login.title",
    "admin.login.subtitle",
    "admin.login.label_username",
    "admin.login.label_password",
    "admin.login.btn_submit",
    "admin.login.error_session",
    "admin.login.error_credentials",
    "admin.login.error_locked",
    // logout
    "admin.logout.success",
    // access
    "admin.access.insufficient_rights",
    // dashboard
    "admin.dashboard.subtitle",
    "admin.dashboard.card_resources",
    "admin.dashboard.th_resource",
    "admin.dashboard.th_key",
    "admin.dashboard.th_permissions",
    "admin.dashboard.th_actions",
    "admin.dashboard.btn_list",
    "admin.dashboard.btn_create",
    "admin.dashboard.see_list",
    "admin.dashboard.empty_title",
    "admin.dashboard.empty_desc",
    "admin.dashboard.kpi_resources",
    "admin.dashboard.kpi_entries",
    "admin.dashboard.kpi_largest",
    "admin.dashboard.th_count",
    // list
    "admin.list.entries_count_one",
    "admin.list.entries_count_many",
    "admin.list.btn_create",
    "admin.list.th_id",
    "admin.list.th_actions",
    "admin.list.bool_true",
    "admin.list.bool_false",
    "admin.list.btn_detail",
    "admin.list.btn_edit",
    "admin.list.btn_delete",
    "admin.list.confirm_delete",
    "admin.list.empty_title",
    "admin.list.empty_desc",
    "admin.list.btn_create_first",
    "admin.list.search_placeholder",
    // create
    "admin.create.title",
    "admin.create.breadcrumb",
    "admin.create.card_info",
    "admin.create.no_fields",
    "admin.create.btn_cancel",
    "admin.create.btn_submit",
    // edit
    "admin.edit.title",
    "admin.edit.breadcrumb",
    "admin.edit.card_info",
    "admin.edit.no_fields",
    "admin.edit.btn_cancel",
    "admin.edit.btn_submit",
    // detail
    "admin.detail.title",
    "admin.detail.breadcrumb",
    "admin.detail.entry_label",
    "admin.detail.btn_list",
    "admin.detail.btn_edit",
    "admin.detail.btn_delete",
    "admin.detail.confirm_delete",
    "admin.detail.btn_reset_password",
    "admin.detail.btn_reset_password_short",
    // delete
    "admin.delete.title",
    "admin.delete.breadcrumb",
    "admin.delete.heading",
    "admin.delete.btn_cancel",
    "admin.delete.btn_confirm",
    // delete.warning
    "admin.delete.not_found",
    "admin.delete.warning.title",
    "admin.delete.warning.desc",
    "admin.delete.warning.of",
    "admin.delete.warning.irreversible",
    // base
    "admin.base.title",
    "admin.base.breadcrumb",
    "admin.base.toggle",
    "admin.base.logout_title",
    "admin.base.theme_toggle",
    "admin.base.back_to_site",
    // flash messages
    "admin.create.success",
    "admin.edit.success",
    "admin.delete.success",
    // history
    "admin.history.subtitle",
    "admin.history.whole_row",
    "admin.history.th_column",
    "admin.history.th_old",
    "admin.history.th_new",
    "admin.history.no_changes",
    "admin.history.all_resources",
    "admin.history.all_actions",
    "admin.history.action_create",
    "admin.history.action_update",
    "admin.history.action_delete",
    "admin.history.btn_filter",
    "admin.history.btn_reset",
    "admin.history.th_date",
    "admin.history.th_account",
    "admin.history.th_resource",
    "admin.history.th_object",
    "admin.history.th_action",
    "admin.history.btn_entry",
    "admin.history.empty",
    "admin.history.badge_create",
    "admin.history.badge_update",
    "admin.history.badge_delete",
    "admin.history.badge_rights",
    "admin.history.badge_rank",
    "admin.history.badge_reset_password",
    // rights grid
    "admin.rights.title",
    "admin.rights.subtitle",
    "admin.rights.go_to_table",
    "admin.rights.toc",
    "admin.rights.btn_cancel",
    "admin.rights.btn_save",
    "admin.rights.th_column",
    "admin.rights.col_read",
    "admin.rights.col_create",
    "admin.rights.col_update",
    // added to existing sections
    "admin.detail.btn_rights",
    "admin.detail.btn_rank",
    "admin.list.bulk_selected",
    "admin.list.bulk_delete",
    "admin.list.filters",
    "admin.list.filters_hide",
    "admin.list.filters_clear",
    "admin.list.select_all",
    "admin.list.row_details",
    "admin.base.dashboard",
    "admin.base.history",
    "admin.dashboard.toggle_template",
];

/// Names of the admin's generated routes, as `{% link %}` takes them:
/// `{% link "admin_detail" resource=…, id=… %}`.
pub mod admin_route {
    /// `{prefix}/{resource}/list`
    pub const LIST: &str = "admin_list";
    /// `{prefix}/{resource}/{id}/detail`
    pub const DETAIL: &str = "admin_detail";
    /// `{prefix}/{resource}/create`
    pub const CREATE: &str = "admin_create";
    /// `{prefix}/{resource}/{id}/edit`
    pub const EDIT: &str = "admin_edit";
    /// `{prefix}/{resource}/{id}/delete`
    pub const DELETE: &str = "admin_delete";
    /// `{prefix}/{resource}/{id}/reset-password` (POST): sends the account's
    /// owner a link to choose a new password
    pub const RESET_PASSWORD: &str = "admin_reset_password";
    /// `{prefix}/{resource}/{id}/rights`: an admin's own rights grid
    pub const RIGHTS: &str = "admin_rights";
    /// `{prefix}/{resource}/rank` (POST): gives the accounts `ids` a rank
    pub const RANK: &str = "admin_rank";
    /// `{prefix}/{resource}/bulk` (POST): a grouped action on the checked rows
    pub const BULK: &str = "admin_bulk";
    /// `{prefix}/history`, filtered by `?resource=&object=&action=&user=`
    pub const HISTORY: &str = "admin_history";
    /// `{prefix}/history/{id}`: one entry and its changes
    pub const HISTORY_ENTRY: &str = "admin_history_entry";
}

/// Names of the forms the admin's pages build, sent back in their hidden
/// `runique_form` field: a form sent to another page than its own is never
/// validated there.
pub mod admin_form {
    pub const CREATE: &str = "admin_create";
    pub const UPDATE: &str = "admin_update";
    pub const DELETE: &str = "admin_delete";
    pub const BULK: &str = "admin_bulk";
    pub const RESET_PASSWORD: &str = "admin_reset_password";
    pub const RANK: &str = "admin_rank";
    pub const RIGHTS: &str = "admin_rights";
}
