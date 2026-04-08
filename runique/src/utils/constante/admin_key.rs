//! Constantes des clés de contexte Tera pour les templates admin (list, form, etc.).

/// Clés de contexte Tera injectées dans les templates admin.
/// Utilisez ces constantes pour surcharger un template sans risquer de typos.
pub mod admin_context {
    /// Clés partagées — injectées dans toutes les vues admin
    pub mod common {
        pub const LANG: &str = "lang";
        pub const SITE_TITLE: &str = "site_title";
        pub const SITE_URL: &str = "site_url";
        pub const RESOURCE: &str = "resource";
        pub const RESOURCE_KEY: &str = "resource_key";
        pub const CURRENT_RESOURCE: &str = "current_resource";
        pub const RESOURCES: &str = "resources";
        pub const REGISTERED_ROLES: &str = "registered_roles";
        pub const ENTRY: &str = "entry";
        pub const OBJECT_ID: &str = "object_id";
        pub const FORM_FIELDS: &str = "form_fields";
        pub const IS_EDIT: &str = "is_edit";
    }

    pub mod permission {
        pub const CAN_CREATE: &str = "can_create";
        pub const CAN_READ: &str = "can_read";
        pub const CAN_UPDATE: &str = "can_update";
        pub const CAN_DELETE: &str = "can_delete";
        pub const CAN_UPDATE_OWN: &str = "can_update_own";
        pub const CAN_DELETE_OWN: &str = "can_delete_own";
        pub const GROUPE_ID: &str = "groupe_id";
        pub const GROUPES: &str = "groupes";
    }

    pub mod etat {}
    /// Template `list` — vue liste d'une ressource
    pub mod list {
        pub use super::common::LANG;
        pub const ENTRIES: &str = "entries";
        pub const TOTAL: &str = "total";
        pub const PAGE: &str = "page";
        pub const PAGE_COUNT: &str = "page_count";
        pub const HAS_PREV: &str = "has_prev";
        pub const HAS_NEXT: &str = "has_next";
        pub const PREV_PAGE: &str = "prev_page";
        pub const NEXT_PAGE: &str = "next_page";
        pub const VISIBLE_COLUMNS: &str = "visible_columns";
        pub const COLUMN_LABELS: &str = "column_labels";
        pub const SORT_BY: &str = "sort_by";
        pub const SORT_DIR: &str = "sort_dir";
        pub const SORT_DIR_TOGGLE: &str = "sort_dir_toggle";
        pub const SEARCH: &str = "search";
        pub const FILTER_VALUES: &str = "filter_values";
        pub const ACTIVE_FILTERS: &str = "active_filters";
        pub const FILTER_QS: &str = "filter_qs";
        pub const FILTER_META: &str = "filter_meta";

        /// Clés obligatoires pour la surcharge de ce template
        pub const REQUIRED: &[&str] = &[
            ENTRIES,
            TOTAL,
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
        ];
    }

    /// Template `create` — formulaire de creation
    pub mod create {
        pub use super::common::{FORM_FIELDS, IS_EDIT, LANG};

        /// Clés obligatoires pour la surcharge de ce template
        pub const REQUIRED: &[&str] = &[FORM_FIELDS];
    }

    /// Template `edit` — formulaire d'edition
    pub mod edit {
        pub use super::common::{FORM_FIELDS, IS_EDIT, LANG, OBJECT_ID};

        /// Clés obligatoires pour la surcharge de ce template
        pub const REQUIRED: &[&str] = &[FORM_FIELDS, OBJECT_ID];
    }

    /// Template `detail` — vue detail d'un objet
    pub mod detail {
        pub use super::common::{ENTRY, OBJECT_ID};

        /// Clés obligatoires pour la surcharge de ce template
        pub const REQUIRED: &[&str] = &[ENTRY, OBJECT_ID];
    }

    /// Template `delete` — confirmation de suppression
    pub mod delete {
        pub use super::common::{ENTRY, OBJECT_ID};

        /// Clés obligatoires pour la surcharge de ce template
        pub const REQUIRED: &[&str] = &[ENTRY, OBJECT_ID];
    }
}

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
    // list
    "admin.list.breadcrumb_admin",
    "admin.list.entries_count",
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
    // flash messages
    "admin.create.success",
    "admin.edit.success",
    "admin.delete.success",
];
