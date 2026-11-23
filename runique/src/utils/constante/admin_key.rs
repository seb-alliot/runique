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

/// The messages of the built-in admin templates, grouped by section: the
/// template variable, then the key of the message.
/// [`insert_admin_messages`](crate::admin::trad::insert_admin_messages) filters
/// this list by the `admin.<section>.` prefix of the key and injects each match
/// into the Tera context under its variable.
///
/// The variable is what a template reads, overridden ones included: it stays
/// the same when a key is renamed.
pub const ADMIN_MESSAGE_KEYS: &[(&str, &str)] = &[
    // login
    (
        "admin_login_title",
        runique_i18n::keys::admin::login::label::TITLE,
    ),
    (
        "admin_login_subtitle",
        runique_i18n::keys::admin::login::label::SUBTITLE,
    ),
    (
        "admin_login_label_username",
        runique_i18n::keys::admin::login::label::LABEL_USERNAME,
    ),
    (
        "admin_login_label_password",
        runique_i18n::keys::admin::login::label::LABEL_PASSWORD,
    ),
    (
        "admin_login_btn_submit",
        runique_i18n::keys::admin::login::label::BTN_SUBMIT,
    ),
    (
        "admin_login_back_to_site",
        runique_i18n::keys::admin::login::label::BACK_TO_SITE,
    ),
    (
        "admin_login_error_session",
        runique_i18n::keys::admin::login::error::SESSION,
    ),
    (
        "admin_login_error_credentials",
        runique_i18n::keys::admin::login::error::CREDENTIALS,
    ),
    (
        "admin_login_error_locked",
        runique_i18n::keys::admin::login::error::LOCKED,
    ),
    // logout
    (
        "admin_logout_success",
        runique_i18n::keys::admin::logout::info::SUCCESS,
    ),
    // access
    (
        "admin_access_insufficient_rights",
        runique_i18n::keys::admin::access::error::INSUFFICIENT_RIGHTS,
    ),
    // dashboard
    (
        "admin_dashboard_subtitle",
        runique_i18n::keys::admin::dashboard::label::SUBTITLE,
    ),
    (
        "admin_dashboard_card_resources",
        runique_i18n::keys::admin::dashboard::label::CARD_RESOURCES,
    ),
    (
        "admin_dashboard_th_resource",
        runique_i18n::keys::admin::dashboard::label::TH_RESOURCE,
    ),
    (
        "admin_dashboard_th_key",
        runique_i18n::keys::admin::dashboard::label::TH_KEY,
    ),
    (
        "admin_dashboard_th_permissions",
        runique_i18n::keys::admin::dashboard::label::TH_PERMISSIONS,
    ),
    (
        "admin_dashboard_th_actions",
        runique_i18n::keys::admin::dashboard::label::TH_ACTIONS,
    ),
    (
        "admin_dashboard_btn_list",
        runique_i18n::keys::admin::dashboard::label::BTN_LIST,
    ),
    (
        "admin_dashboard_btn_create",
        runique_i18n::keys::admin::dashboard::label::BTN_CREATE,
    ),
    (
        "admin_dashboard_see_list",
        runique_i18n::keys::admin::dashboard::label::SEE_LIST,
    ),
    (
        "admin_dashboard_empty_title",
        runique_i18n::keys::admin::dashboard::label::EMPTY_TITLE,
    ),
    (
        "admin_dashboard_empty_desc",
        runique_i18n::keys::admin::dashboard::label::EMPTY_DESC,
    ),
    (
        "admin_dashboard_kpi_resources",
        runique_i18n::keys::admin::dashboard::label::KPI_RESOURCES,
    ),
    (
        "admin_dashboard_kpi_entries",
        runique_i18n::keys::admin::dashboard::label::KPI_ENTRIES,
    ),
    (
        "admin_dashboard_kpi_largest",
        runique_i18n::keys::admin::dashboard::label::KPI_LARGEST,
    ),
    (
        "admin_dashboard_th_count",
        runique_i18n::keys::admin::dashboard::label::TH_COUNT,
    ),
    // list
    (
        "admin_list_entries_count_one",
        runique_i18n::keys::admin::list::label::ENTRIES_COUNT_ONE,
    ),
    (
        "admin_list_entries_count_many",
        runique_i18n::keys::admin::list::label::ENTRIES_COUNT_MANY,
    ),
    (
        "admin_list_btn_create",
        runique_i18n::keys::admin::list::label::BTN_CREATE,
    ),
    (
        "admin_list_rank_label",
        runique_i18n::keys::admin::list::label::RANK_LABEL,
    ),
    (
        "admin_list_groups_label",
        runique_i18n::keys::admin::list::label::GROUPS_LABEL,
    ),
    (
        "admin_list_th_id",
        runique_i18n::keys::admin::list::label::TH_ID,
    ),
    (
        "admin_list_th_actions",
        runique_i18n::keys::admin::list::label::TH_ACTIONS,
    ),
    (
        "admin_list_bool_true",
        runique_i18n::keys::admin::list::label::BOOL_TRUE,
    ),
    (
        "admin_list_bool_false",
        runique_i18n::keys::admin::list::label::BOOL_FALSE,
    ),
    (
        "admin_list_btn_detail",
        runique_i18n::keys::admin::list::label::BTN_DETAIL,
    ),
    (
        "admin_list_btn_edit",
        runique_i18n::keys::admin::list::label::BTN_EDIT,
    ),
    (
        "admin_list_btn_delete",
        runique_i18n::keys::admin::list::label::BTN_DELETE,
    ),
    (
        "admin_list_confirm_delete",
        runique_i18n::keys::admin::list::label::CONFIRM_DELETE,
    ),
    (
        "admin_list_empty_title",
        runique_i18n::keys::admin::list::label::EMPTY_TITLE,
    ),
    (
        "admin_list_empty_desc",
        runique_i18n::keys::admin::list::label::EMPTY_DESC,
    ),
    (
        "admin_list_btn_create_first",
        runique_i18n::keys::admin::list::label::BTN_CREATE_FIRST,
    ),
    (
        "admin_list_search_placeholder",
        runique_i18n::keys::admin::list::label::SEARCH_PLACEHOLDER,
    ),
    (
        "admin_list_per_page",
        runique_i18n::keys::admin::list::label::PER_PAGE,
    ),
    (
        "admin_list_filter_per_page",
        runique_i18n::keys::admin::list::label::FILTER_PER_PAGE,
    ),
    // create
    (
        "admin_create_title",
        runique_i18n::keys::admin::create::label::TITLE,
    ),
    (
        "admin_create_breadcrumb",
        runique_i18n::keys::admin::create::label::BREADCRUMB,
    ),
    (
        "admin_create_card_info",
        runique_i18n::keys::admin::create::label::CARD_INFO,
    ),
    (
        "admin_create_no_fields",
        runique_i18n::keys::admin::create::label::NO_FIELDS,
    ),
    (
        "admin_create_btn_cancel",
        runique_i18n::keys::admin::create::label::BTN_CANCEL,
    ),
    (
        "admin_create_btn_submit",
        runique_i18n::keys::admin::create::label::BTN_SUBMIT,
    ),
    // edit
    (
        "admin_edit_title",
        runique_i18n::keys::admin::edit::label::TITLE,
    ),
    (
        "admin_edit_breadcrumb",
        runique_i18n::keys::admin::edit::label::BREADCRUMB,
    ),
    (
        "admin_edit_card_info",
        runique_i18n::keys::admin::edit::label::CARD_INFO,
    ),
    (
        "admin_edit_no_fields",
        runique_i18n::keys::admin::edit::label::NO_FIELDS,
    ),
    (
        "admin_edit_btn_cancel",
        runique_i18n::keys::admin::edit::label::BTN_CANCEL,
    ),
    (
        "admin_edit_btn_submit",
        runique_i18n::keys::admin::edit::label::BTN_SUBMIT,
    ),
    // detail
    (
        "admin_detail_title",
        runique_i18n::keys::admin::detail::label::TITLE,
    ),
    (
        "admin_detail_breadcrumb",
        runique_i18n::keys::admin::detail::label::BREADCRUMB,
    ),
    (
        "admin_detail_entry_label",
        runique_i18n::keys::admin::detail::label::ENTRY_LABEL,
    ),
    (
        "admin_detail_btn_list",
        runique_i18n::keys::admin::detail::label::BTN_LIST,
    ),
    (
        "admin_detail_btn_edit",
        runique_i18n::keys::admin::detail::label::BTN_EDIT,
    ),
    (
        "admin_detail_btn_delete",
        runique_i18n::keys::admin::detail::label::BTN_DELETE,
    ),
    (
        "admin_detail_confirm_delete",
        runique_i18n::keys::admin::detail::label::CONFIRM_DELETE,
    ),
    (
        "admin_detail_btn_reset_password",
        runique_i18n::keys::admin::detail::label::BTN_RESET_PASSWORD,
    ),
    (
        "admin_detail_btn_reset_password_short",
        runique_i18n::keys::admin::detail::label::BTN_RESET_PASSWORD_SHORT,
    ),
    // delete
    (
        "admin_delete_title",
        runique_i18n::keys::admin::delete::label::TITLE,
    ),
    (
        "admin_delete_breadcrumb",
        runique_i18n::keys::admin::delete::label::BREADCRUMB,
    ),
    (
        "admin_delete_heading",
        runique_i18n::keys::admin::delete::label::HEADING,
    ),
    (
        "admin_delete_btn_cancel",
        runique_i18n::keys::admin::delete::label::BTN_CANCEL,
    ),
    (
        "admin_delete_btn_confirm",
        runique_i18n::keys::admin::delete::label::BTN_CONFIRM,
    ),
    // delete.warning
    (
        "admin_delete_not_found",
        runique_i18n::keys::admin::delete::error::NOT_FOUND,
    ),
    (
        "admin_delete_warning_title",
        runique_i18n::keys::admin::delete::label::WARNING_TITLE,
    ),
    (
        "admin_delete_warning_desc",
        runique_i18n::keys::admin::delete::label::WARNING_DESC,
    ),
    (
        "admin_delete_warning_of",
        runique_i18n::keys::admin::delete::label::WARNING_OF,
    ),
    (
        "admin_delete_warning_irreversible",
        runique_i18n::keys::admin::delete::label::WARNING_IRREVERSIBLE,
    ),
    // base
    (
        "admin_base_title",
        runique_i18n::keys::admin::base::label::TITLE,
    ),
    (
        "admin_base_breadcrumb",
        runique_i18n::keys::admin::base::label::BREADCRUMB,
    ),
    (
        "admin_base_toggle",
        runique_i18n::keys::admin::base::label::TOGGLE,
    ),
    (
        "admin_base_logout_title",
        runique_i18n::keys::admin::base::label::LOGOUT_TITLE,
    ),
    (
        "admin_base_theme_toggle",
        runique_i18n::keys::admin::base::label::THEME_TOGGLE,
    ),
    (
        "admin_base_back_to_site",
        runique_i18n::keys::admin::base::label::BACK_TO_SITE,
    ),
    // flash messages
    (
        "admin_create_success",
        runique_i18n::keys::admin::create::info::SUCCESS,
    ),
    (
        "admin_edit_success",
        runique_i18n::keys::admin::edit::info::SUCCESS,
    ),
    (
        "admin_delete_success",
        runique_i18n::keys::admin::delete::info::SUCCESS,
    ),
    // history
    (
        "admin_history_subtitle",
        runique_i18n::keys::admin::history::label::SUBTITLE,
    ),
    (
        "admin_history_whole_row",
        runique_i18n::keys::admin::history::label::WHOLE_ROW,
    ),
    (
        "admin_history_th_column",
        runique_i18n::keys::admin::history::label::TH_COLUMN,
    ),
    (
        "admin_history_th_old",
        runique_i18n::keys::admin::history::label::TH_OLD,
    ),
    (
        "admin_history_th_new",
        runique_i18n::keys::admin::history::label::TH_NEW,
    ),
    (
        "admin_history_no_changes",
        runique_i18n::keys::admin::history::label::NO_CHANGES,
    ),
    (
        "admin_history_all_resources",
        runique_i18n::keys::admin::history::label::ALL_RESOURCES,
    ),
    (
        "admin_history_all_actions",
        runique_i18n::keys::admin::history::label::ALL_ACTIONS,
    ),
    (
        "admin_history_action_create",
        runique_i18n::keys::admin::history::label::ACTION_CREATE,
    ),
    (
        "admin_history_action_update",
        runique_i18n::keys::admin::history::label::ACTION_UPDATE,
    ),
    (
        "admin_history_action_delete",
        runique_i18n::keys::admin::history::label::ACTION_DELETE,
    ),
    (
        "admin_history_btn_filter",
        runique_i18n::keys::admin::history::label::BTN_FILTER,
    ),
    (
        "admin_history_btn_reset",
        runique_i18n::keys::admin::history::label::BTN_RESET,
    ),
    (
        "admin_history_th_date",
        runique_i18n::keys::admin::history::label::TH_DATE,
    ),
    (
        "admin_history_th_account",
        runique_i18n::keys::admin::history::label::TH_ACCOUNT,
    ),
    (
        "admin_history_th_resource",
        runique_i18n::keys::admin::history::label::TH_RESOURCE,
    ),
    (
        "admin_history_th_object",
        runique_i18n::keys::admin::history::label::TH_OBJECT,
    ),
    (
        "admin_history_th_action",
        runique_i18n::keys::admin::history::label::TH_ACTION,
    ),
    (
        "admin_history_btn_entry",
        runique_i18n::keys::admin::history::label::BTN_ENTRY,
    ),
    (
        "admin_history_empty",
        runique_i18n::keys::admin::history::label::EMPTY,
    ),
    (
        "admin_history_badge_create",
        runique_i18n::keys::admin::history::label::BADGE_CREATE,
    ),
    (
        "admin_history_badge_update",
        runique_i18n::keys::admin::history::label::BADGE_UPDATE,
    ),
    (
        "admin_history_badge_delete",
        runique_i18n::keys::admin::history::label::BADGE_DELETE,
    ),
    (
        "admin_history_badge_rights",
        runique_i18n::keys::admin::history::label::BADGE_RIGHTS,
    ),
    (
        "admin_history_badge_rank",
        runique_i18n::keys::admin::history::label::BADGE_RANK,
    ),
    (
        "admin_history_badge_reset_password",
        runique_i18n::keys::admin::history::label::BADGE_RESET_PASSWORD,
    ),
    // rights grid
    (
        "admin_rights_title",
        runique_i18n::keys::admin::rights::label::TITLE,
    ),
    (
        "admin_rights_subtitle",
        runique_i18n::keys::admin::rights::label::SUBTITLE,
    ),
    (
        "admin_rights_group_subtitle",
        runique_i18n::keys::admin::rights::label::GROUP_SUBTITLE,
    ),
    (
        "admin_rights_group_label",
        runique_i18n::keys::admin::rights::label::GROUP_LABEL,
    ),
    (
        "admin_rights_btn_show",
        runique_i18n::keys::admin::rights::label::BTN_SHOW,
    ),
    (
        "admin_rights_group_sheet",
        runique_i18n::keys::admin::rights::label::GROUP_SHEET,
    ),
    (
        "admin_rights_go_to_table",
        runique_i18n::keys::admin::rights::label::GO_TO_TABLE,
    ),
    (
        "admin_rights_toc",
        runique_i18n::keys::admin::rights::label::TOC,
    ),
    (
        "admin_rights_btn_cancel",
        runique_i18n::keys::admin::rights::label::BTN_CANCEL,
    ),
    (
        "admin_rights_btn_save",
        runique_i18n::keys::admin::rights::label::BTN_SAVE,
    ),
    (
        "admin_rights_th_column",
        runique_i18n::keys::admin::rights::label::TH_COLUMN,
    ),
    (
        "admin_rights_col_read",
        runique_i18n::keys::admin::rights::label::COL_READ,
    ),
    (
        "admin_rights_col_create",
        runique_i18n::keys::admin::rights::label::COL_CREATE,
    ),
    (
        "admin_rights_col_update",
        runique_i18n::keys::admin::rights::label::COL_UPDATE,
    ),
    // added to existing sections
    (
        "admin_detail_btn_rights",
        runique_i18n::keys::admin::detail::label::BTN_RIGHTS,
    ),
    (
        "admin_detail_btn_rank",
        runique_i18n::keys::admin::detail::label::BTN_RANK,
    ),
    (
        "admin_detail_groups_title",
        runique_i18n::keys::admin::detail::label::GROUPS_TITLE,
    ),
    (
        "admin_detail_btn_save_groups",
        runique_i18n::keys::admin::detail::label::BTN_SAVE_GROUPS,
    ),
    (
        "admin_list_btn_rights",
        runique_i18n::keys::admin::list::label::BTN_RIGHTS,
    ),
    (
        "admin_list_bulk_selected",
        runique_i18n::keys::admin::list::label::BULK_SELECTED,
    ),
    (
        "admin_list_bulk_delete",
        runique_i18n::keys::admin::list::label::BULK_DELETE,
    ),
    (
        "admin_list_filters",
        runique_i18n::keys::admin::list::label::FILTERS,
    ),
    (
        "admin_list_filters_hide",
        runique_i18n::keys::admin::list::label::FILTERS_HIDE,
    ),
    (
        "admin_list_filters_clear",
        runique_i18n::keys::admin::list::label::FILTERS_CLEAR,
    ),
    (
        "admin_list_select_all",
        runique_i18n::keys::admin::list::label::SELECT_ALL,
    ),
    (
        "admin_list_row_details",
        runique_i18n::keys::admin::list::label::ROW_DETAILS,
    ),
    (
        "admin_base_dashboard",
        runique_i18n::keys::admin::base::label::DASHBOARD,
    ),
    (
        "admin_base_history",
        runique_i18n::keys::admin::base::label::HISTORY,
    ),
    (
        "admin_dashboard_toggle_template",
        runique_i18n::keys::admin::dashboard::label::TOGGLE_TEMPLATE,
    ),
    (
        "admin_dashboard_my_groups_title",
        runique_i18n::keys::admin::dashboard::label::MY_GROUPS_TITLE,
    ),
    (
        "admin_transfer_title",
        runique_i18n::keys::admin::transfer::label::TITLE,
    ),
    (
        "admin_transfer_from",
        runique_i18n::keys::admin::transfer::label::FROM,
    ),
    (
        "admin_transfer_to",
        runique_i18n::keys::admin::transfer::label::TO,
    ),
    (
        "admin_transfer_losses",
        runique_i18n::keys::admin::transfer::label::LOSSES,
    ),
    (
        "admin_transfer_gained_right",
        runique_i18n::keys::admin::transfer::label::GAINED_RIGHT,
    ),
    (
        "admin_transfer_lost_right",
        runique_i18n::keys::admin::transfer::label::LOST_RIGHT,
    ),
    (
        "admin_transfer_no_loss",
        runique_i18n::keys::admin::transfer::label::NO_LOSS,
    ),
    (
        "admin_transfer_btn_confirm",
        runique_i18n::keys::admin::transfer::label::BTN_CONFIRM,
    ),
    (
        "admin_transfer_btn_cancel",
        runique_i18n::keys::admin::transfer::label::BTN_CANCEL,
    ),
    (
        "admin_detail_transfer_title",
        runique_i18n::keys::admin::detail::label::TRANSFER_TITLE,
    ),
    (
        "admin_detail_btn_transfer",
        runique_i18n::keys::admin::detail::label::BTN_TRANSFER,
    ),
    (
        "admin_list_responsable_pending",
        runique_i18n::keys::admin::list::label::RESPONSABLE_PENDING,
    ),
    (
        "admin_list_taken_by",
        runique_i18n::keys::admin::list::label::TAKEN_BY,
    ),
    (
        "admin_history_badge_groups",
        runique_i18n::keys::admin::history::label::BADGE_GROUPS,
    ),
    (
        "admin_history_badge_transfer",
        runique_i18n::keys::admin::history::label::BADGE_TRANSFER,
    ),
    (
        "admin_dashboard_my_groups_staff",
        runique_i18n::keys::admin::dashboard::label::MY_GROUPS_STAFF,
    ),
    (
        "admin_dashboard_my_groups_none",
        runique_i18n::keys::admin::dashboard::label::MY_GROUPS_NONE,
    ),
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
