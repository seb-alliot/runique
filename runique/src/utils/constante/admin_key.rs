//! Admin keys: framework table keys, route and form names, message keys.

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
    "admin.rights.group_subtitle",
    "admin.rights.group_label",
    "admin.rights.btn_show",
    "admin.rights.group_sheet",
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
    "admin.detail.groups_title",
    "admin.detail.btn_save_groups",
    "admin.list.btn_rights",
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
    "admin.dashboard.my_groups_title",
    "admin.transfer.title",
    "admin.transfer.from",
    "admin.transfer.to",
    "admin.transfer.losses",
    "admin.transfer.gained_right",
    "admin.transfer.lost_right",
    "admin.transfer.no_loss",
    "admin.transfer.btn_confirm",
    "admin.transfer.btn_cancel",
    "admin.detail.transfer_title",
    "admin.detail.btn_transfer",
    "admin.list.responsable_pending",
    "admin.list.taken_by",
    "admin.history.badge_groups",
    "admin.history.badge_transfer",
    "admin.dashboard.my_groups_staff",
    "admin.dashboard.my_groups_none",
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
    /// `{prefix}/groupes/rights?groupe={id}` (GET): the group picked in the
    /// rights page's selector, redirected to its grid
    pub const GROUP_SELECT: &str = "admin_group_select";
    /// `{prefix}/{resource}/{id}/groups` (POST): an account's groups
    pub const GROUPS: &str = "admin_groups";
    /// `{prefix}/groupes/{id}/transfer`: what handing a group over does
    /// (GET), then the transfer (POST)
    pub const TRANSFER: &str = "admin_transfer";
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
    pub const GROUPS: &str = "admin_groups";
    pub const TRANSFER: &str = "admin_transfer";
}
