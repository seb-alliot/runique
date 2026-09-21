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
        /// Roles registered via `admin!{}` (see `admin::helper::roles::get_roles()`).
        pub const REGISTERED_ROLES: &str = "registered_roles";
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

    /// Keys used by the droit (permission) admin form — each maps to a CRUD flag
    /// or to the parent group scoping the droit.
    pub mod permission {
        /// Whether the group can create objects of the resource.
        pub const CAN_CREATE: &str = "can_create";
        /// Whether the group can read/list objects of the resource.
        pub const CAN_READ: &str = "can_read";
        /// Whether the group can update any object of the resource.
        pub const CAN_UPDATE: &str = "can_update";
        /// Whether the group can delete any object of the resource.
        pub const CAN_DELETE: &str = "can_delete";
        /// Whether the group can update only the objects it owns.
        pub const CAN_UPDATE_OWN: &str = "can_update_own";
        /// Whether the group can delete only the objects it owns.
        pub const CAN_DELETE_OWN: &str = "can_delete_own";
        /// Form field holding the id of the parent group a droit belongs to.
        pub const GROUPE_ID: &str = "groupe_id";
        /// Resource key of the `groupes` admin resource — droits are a scoped
        /// child of it (`/groupes/{id}/droits/...`), see `ParentScope`.
        pub const GROUPES: &str = "groupes";
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
        /// Column name → i18n label map, falling back to `permission.col.*` keys.
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

        /// Mandatory keys for overriding this template
        pub const REQUIRED: &[&str] = &[
            ENTRIES,
            PAGE,
            PAGE_COUNT,
            HAS_PREV,
            HAS_NEXT,
            PREV_PAGE,
            NEXT_PAGE,
            VISIBLE_COLUMNS,
            SORT_BY,
            SORT_DIR,
            SORT_DIR_TOGGLE,
            SEARCH,
            RETURN_QS,
            GROUP_ACTIONS,
        ];
    }

    /// `create` template — creation form
    pub mod create {
        pub use super::common::{FORM_FIELDS, IS_EDIT, LANG};

        /// Mandatory keys for overriding this template
        pub const REQUIRED: &[&str] = &[FORM_FIELDS];
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

        /// Mandatory keys for overriding this template
        pub const REQUIRED: &[&str] = &[FORM_FIELDS, OBJECT_ID, RETURN_QS];
    }

    /// `detail` template — object detail view
    pub mod detail {
        pub use super::common::{ENTRY, OBJECT_ID};

        /// Mandatory keys for overriding this template
        pub const REQUIRED: &[&str] = &[ENTRY, OBJECT_ID];
    }

    /// `delete` template — deletion confirmation
    pub mod delete {
        pub use super::common::{ENTRY, OBJECT_ID};

        /// Mandatory keys for overriding this template
        pub const REQUIRED: &[&str] = &[ENTRY, OBJECT_ID];
    }

    /// `bulk_edit` template — multi-row batch edit form
    pub mod bulk_edit {
        pub use super::common::{FORM_FIELDS, LANG};
        /// Number of selected rows being edited.
        pub const BULK_COUNT: &str = "bulk_count";
        /// Comma-separated IDs of the selected rows — value of the hidden `ids` field.
        pub const BULK_IDS: &str = "bulk_ids";

        /// Mandatory keys for overriding this template
        pub const REQUIRED: &[&str] = &[FORM_FIELDS, BULK_COUNT, BULK_IDS];
    }
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
    "admin.access.no_auth_handler",
    "admin.access.insufficient_rights",
    // dashboard
    "admin.dashboard.title",
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
    "admin.list.breadcrumb_admin",
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
    // Permission html
    "permission.col.id",
    "permission.col.can_create",
    "permission.col.can_delete",
    "permission.col.can_delete_own",
    "permission.col.can_read",
    "permission.col.can_update",
    "permission.col.can_update_own",
    "permission.col.groupes",
    "permission.col.resource_key",
    "permission.col.actions",
];
