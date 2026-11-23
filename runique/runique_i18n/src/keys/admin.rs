//! Keys of `admin`.
pub mod access {
    pub mod error {
        pub const ACTION_NOT_ALLOWED: &str = "admin.access.error.action_not_allowed";
        pub const INSUFFICIENT_RIGHTS: &str = "admin.access.error.insufficient_rights";
        pub const STILL_RESPONSABLE: &str = "admin.access.error.still_responsable";
    }
}
pub mod base {
    pub mod label {
        pub const BACK_TO_SITE: &str = "admin.base.label.back_to_site";
        pub const BREADCRUMB: &str = "admin.base.label.breadcrumb";
        pub const DASHBOARD: &str = "admin.base.label.dashboard";
        pub const HISTORY: &str = "admin.base.label.history";
        pub const LOGOUT_TITLE: &str = "admin.base.label.logout_title";
        pub const THEME_TOGGLE: &str = "admin.base.label.theme_toggle";
        pub const TITLE: &str = "admin.base.label.title";
        pub const TOGGLE: &str = "admin.base.label.toggle";
    }
}
pub mod bulk {
    pub mod error {
        pub const NO_SELECTION: &str = "admin.bulk.error.no_selection";
    }
    pub mod info {
        pub const DELETE_SUCCESS: &str = "admin.bulk.info.delete_success";
        pub const UPDATE_SUCCESS: &str = "admin.bulk.info.update_success";
    }
}
pub mod common {
    pub mod label {
        pub const PASSWORD: &str = "admin.common.label.password";
        pub const USERNAME: &str = "admin.common.label.username";
    }
}
pub mod create {
    pub mod info {
        pub const SUCCESS: &str = "admin.create.info.success";
    }
    pub mod label {
        pub const BREADCRUMB: &str = "admin.create.label.breadcrumb";
        pub const BTN_CANCEL: &str = "admin.create.label.btn_cancel";
        pub const BTN_SUBMIT: &str = "admin.create.label.btn_submit";
        pub const CARD_INFO: &str = "admin.create.label.card_info";
        pub const NO_FIELDS: &str = "admin.create.label.no_fields";
        pub const TITLE: &str = "admin.create.label.title";
    }
}
pub mod dashboard {
    pub mod label {
        pub const BTN_CREATE: &str = "admin.dashboard.label.btn_create";
        pub const BTN_LIST: &str = "admin.dashboard.label.btn_list";
        pub const CARD_RESOURCES: &str = "admin.dashboard.label.card_resources";
        pub const EMPTY_DESC: &str = "admin.dashboard.label.empty_desc";
        pub const EMPTY_TITLE: &str = "admin.dashboard.label.empty_title";
        pub const KPI_ENTRIES: &str = "admin.dashboard.label.kpi_entries";
        pub const KPI_LARGEST: &str = "admin.dashboard.label.kpi_largest";
        pub const KPI_RESOURCES: &str = "admin.dashboard.label.kpi_resources";
        pub const MY_GROUPS_NONE: &str = "admin.dashboard.label.my_groups_none";
        pub const MY_GROUPS_STAFF: &str = "admin.dashboard.label.my_groups_staff";
        pub const MY_GROUPS_TITLE: &str = "admin.dashboard.label.my_groups_title";
        pub const SEE_LIST: &str = "admin.dashboard.label.see_list";
        pub const SUBTITLE: &str = "admin.dashboard.label.subtitle";
        pub const TH_ACTIONS: &str = "admin.dashboard.label.th_actions";
        pub const TH_COUNT: &str = "admin.dashboard.label.th_count";
        pub const TH_KEY: &str = "admin.dashboard.label.th_key";
        pub const TH_PERMISSIONS: &str = "admin.dashboard.label.th_permissions";
        pub const TH_RESOURCE: &str = "admin.dashboard.label.th_resource";
        pub const TOGGLE_TEMPLATE: &str = "admin.dashboard.label.toggle_template";
    }
}
pub mod delete {
    pub mod error {
        pub const NOT_FOUND: &str = "admin.delete.error.not_found";
    }
    pub mod info {
        pub const SUCCESS: &str = "admin.delete.info.success";
    }
    pub mod label {
        pub const BREADCRUMB: &str = "admin.delete.label.breadcrumb";
        pub const BTN_CANCEL: &str = "admin.delete.label.btn_cancel";
        pub const BTN_CONFIRM: &str = "admin.delete.label.btn_confirm";
        pub const HEADING: &str = "admin.delete.label.heading";
        pub const TITLE: &str = "admin.delete.label.title";
        pub const WARNING_DESC: &str = "admin.delete.label.warning_desc";
        pub const WARNING_IRREVERSIBLE: &str = "admin.delete.label.warning_irreversible";
        pub const WARNING_OF: &str = "admin.delete.label.warning_of";
        pub const WARNING_TITLE: &str = "admin.delete.label.warning_title";
    }
}
pub mod detail {
    pub mod label {
        pub const BREADCRUMB: &str = "admin.detail.label.breadcrumb";
        pub const BTN_DELETE: &str = "admin.detail.label.btn_delete";
        pub const BTN_EDIT: &str = "admin.detail.label.btn_edit";
        pub const BTN_LIST: &str = "admin.detail.label.btn_list";
        pub const BTN_RANK: &str = "admin.detail.label.btn_rank";
        pub const BTN_RESET_PASSWORD: &str = "admin.detail.label.btn_reset_password";
        pub const BTN_RESET_PASSWORD_SHORT: &str = "admin.detail.label.btn_reset_password_short";
        pub const BTN_RIGHTS: &str = "admin.detail.label.btn_rights";
        pub const BTN_SAVE_GROUPS: &str = "admin.detail.label.btn_save_groups";
        pub const BTN_TRANSFER: &str = "admin.detail.label.btn_transfer";
        pub const CONFIRM_DELETE: &str = "admin.detail.label.confirm_delete";
        pub const ENTRY_LABEL: &str = "admin.detail.label.entry_label";
        pub const GROUPS_TITLE: &str = "admin.detail.label.groups_title";
        pub const TITLE: &str = "admin.detail.label.title";
        pub const TRANSFER_TITLE: &str = "admin.detail.label.transfer_title";
    }
}
pub mod edit {
    pub mod info {
        pub const SUCCESS: &str = "admin.edit.info.success";
    }
    pub mod label {
        pub const BREADCRUMB: &str = "admin.edit.label.breadcrumb";
        pub const BTN_CANCEL: &str = "admin.edit.label.btn_cancel";
        pub const BTN_SUBMIT: &str = "admin.edit.label.btn_submit";
        pub const CARD_INFO: &str = "admin.edit.label.card_info";
        pub const NO_FIELDS: &str = "admin.edit.label.no_fields";
        pub const TITLE: &str = "admin.edit.label.title";
    }
}
pub mod history {
    pub mod label {
        pub const ACTION_CREATE: &str = "admin.history.label.action_create";
        pub const ACTION_DELETE: &str = "admin.history.label.action_delete";
        pub const ACTION_UPDATE: &str = "admin.history.label.action_update";
        pub const ALL_ACTIONS: &str = "admin.history.label.all_actions";
        pub const ALL_RESOURCES: &str = "admin.history.label.all_resources";
        pub const BADGE_CREATE: &str = "admin.history.label.badge_create";
        pub const BADGE_DELETE: &str = "admin.history.label.badge_delete";
        pub const BADGE_GROUPS: &str = "admin.history.label.badge_groups";
        pub const BADGE_RANK: &str = "admin.history.label.badge_rank";
        pub const BADGE_RESET_PASSWORD: &str = "admin.history.label.badge_reset_password";
        pub const BADGE_RIGHTS: &str = "admin.history.label.badge_rights";
        pub const BADGE_TRANSFER: &str = "admin.history.label.badge_transfer";
        pub const BADGE_UPDATE: &str = "admin.history.label.badge_update";
        pub const BTN_ENTRY: &str = "admin.history.label.btn_entry";
        pub const BTN_FILTER: &str = "admin.history.label.btn_filter";
        pub const BTN_RESET: &str = "admin.history.label.btn_reset";
        pub const EMPTY: &str = "admin.history.label.empty";
        pub const NO_CHANGES: &str = "admin.history.label.no_changes";
        pub const SUBTITLE: &str = "admin.history.label.subtitle";
        pub const TH_ACCOUNT: &str = "admin.history.label.th_account";
        pub const TH_ACTION: &str = "admin.history.label.th_action";
        pub const TH_COLUMN: &str = "admin.history.label.th_column";
        pub const TH_DATE: &str = "admin.history.label.th_date";
        pub const TH_NEW: &str = "admin.history.label.th_new";
        pub const TH_OBJECT: &str = "admin.history.label.th_object";
        pub const TH_OLD: &str = "admin.history.label.th_old";
        pub const TH_RESOURCE: &str = "admin.history.label.th_resource";
        pub const WHOLE_ROW: &str = "admin.history.label.whole_row";
    }
}
pub mod list {
    pub mod label {
        pub const ACTION_ACTIVATE: &str = "admin.list.label.action_activate";
        pub const ACTION_DEACTIVATE: &str = "admin.list.label.action_deactivate";
        pub const BOOL_FALSE: &str = "admin.list.label.bool_false";
        pub const BOOL_TRUE: &str = "admin.list.label.bool_true";
        pub const BTN_CREATE: &str = "admin.list.label.btn_create";
        pub const BTN_CREATE_FIRST: &str = "admin.list.label.btn_create_first";
        pub const BTN_DELETE: &str = "admin.list.label.btn_delete";
        pub const BTN_DETAIL: &str = "admin.list.label.btn_detail";
        pub const BTN_EDIT: &str = "admin.list.label.btn_edit";
        pub const BTN_RIGHTS: &str = "admin.list.label.btn_rights";
        pub const BULK_DELETE: &str = "admin.list.label.bulk_delete";
        pub const BULK_SELECTED: &str = "admin.list.label.bulk_selected";
        pub const CONFIRM_DELETE: &str = "admin.list.label.confirm_delete";
        pub const EMPTY_DESC: &str = "admin.list.label.empty_desc";
        pub const EMPTY_TITLE: &str = "admin.list.label.empty_title";
        pub const ENTRIES_COUNT_MANY: &str = "admin.list.label.entries_count_many";
        pub const ENTRIES_COUNT_ONE: &str = "admin.list.label.entries_count_one";
        pub const FILTER_PER_PAGE: &str = "admin.list.label.filter_per_page";
        pub const FILTERS: &str = "admin.list.label.filters";
        pub const FILTERS_CLEAR: &str = "admin.list.label.filters_clear";
        pub const FILTERS_HIDE: &str = "admin.list.label.filters_hide";
        pub const GROUPS_LABEL: &str = "admin.list.label.groups_label";
        pub const PER_PAGE: &str = "admin.list.label.per_page";
        pub const RANK_LABEL: &str = "admin.list.label.rank_label";
        pub const RESPONSABLE_PENDING: &str = "admin.list.label.responsable_pending";
        pub const ROW_DETAILS: &str = "admin.list.label.row_details";
        pub const SEARCH_PLACEHOLDER: &str = "admin.list.label.search_placeholder";
        pub const SELECT_ALL: &str = "admin.list.label.select_all";
        pub const TAKEN_BY: &str = "admin.list.label.taken_by";
        pub const TH_ACTIONS: &str = "admin.list.label.th_actions";
        pub const TH_ID: &str = "admin.list.label.th_id";
    }
}
pub mod log {
    pub const BOOT_ADMIN: &str = "admin.log.boot_admin";
    pub const BOOT_MODEL: &str = "admin.log.boot_model";
    pub const BOOT_NO_GUARD: &str = "admin.log.boot_no_guard";
    pub const BOOT_OPTIONS_WITHOUT_EFFECT: &str = "admin.log.boot_options_without_effect";
    pub const BOOT_ROUTE: &str = "admin.log.boot_route";
    pub const BOOT_STYLESHEETS_MISSING: &str = "admin.log.boot_stylesheets_missing";
    pub const BULK_DONE: &str = "admin.log.bulk_done";
    pub const BULK_REFUSED: &str = "admin.log.bulk_refused";
    pub const LIST: &str = "admin.log.list";
    pub const OUTCOME_DELETE_REFUSED: &str = "admin.log.outcome_delete_refused";
    pub const OUTCOME_DELETED: &str = "admin.log.outcome_deleted";
    pub const OUTCOME_SAVE_REFUSED: &str = "admin.log.outcome_save_refused";
    pub const OUTCOME_SAVED: &str = "admin.log.outcome_saved";
    pub const WRITE: &str = "admin.log.write";
}
pub mod login {
    pub mod error {
        pub const CREDENTIALS: &str = "admin.login.error.credentials";
        pub const LOCKED: &str = "admin.login.error.locked";
        pub const SESSION: &str = "admin.login.error.session";
    }
    pub mod label {
        pub const BACK_TO_SITE: &str = "admin.login.label.back_to_site";
        pub const BTN_SUBMIT: &str = "admin.login.label.btn_submit";
        pub const LABEL_PASSWORD: &str = "admin.login.label.label_password";
        pub const LABEL_USERNAME: &str = "admin.login.label.label_username";
        pub const SUBTITLE: &str = "admin.login.label.subtitle";
        pub const TITLE: &str = "admin.login.label.title";
    }
}
pub mod logout {
    pub mod info {
        pub const SUCCESS: &str = "admin.logout.info.success";
    }
}
pub mod rank {
    pub mod info {
        pub const SUCCESS: &str = "admin.rank.info.success";
    }
    pub mod label {
        pub const ADMIN: &str = "admin.rank.label.admin";
        pub const DIRIGEANT: &str = "admin.rank.label.dirigeant";
        pub const LAMBDA: &str = "admin.rank.label.lambda";
        pub const SUPERUSER: &str = "admin.rank.label.superuser";
    }
}
pub mod reset_password {
    pub mod error {
        pub const NO_MAILER: &str = "admin.reset_password.error.no_mailer";
        pub const UNAVAILABLE: &str = "admin.reset_password.error.unavailable";
    }
    pub mod info {
        pub const SUCCESS_EMAIL: &str = "admin.reset_password.info.success_email";
    }
    pub mod label {
        pub const EMAIL_BODY: &str = "admin.reset_password.label.email_body";
        pub const EMAIL_BTN: &str = "admin.reset_password.label.email_btn";
        pub const EMAIL_GREETING: &str = "admin.reset_password.label.email_greeting";
        pub const EMAIL_IGNORE: &str = "admin.reset_password.label.email_ignore";
        pub const EMAIL_SUBJECT: &str = "admin.reset_password.label.email_subject";
        pub const EMAIL_TITLE: &str = "admin.reset_password.label.email_title";
    }
}
pub mod rights {
    pub mod label {
        pub const BTN_CANCEL: &str = "admin.rights.label.btn_cancel";
        pub const BTN_SAVE: &str = "admin.rights.label.btn_save";
        pub const BTN_SHOW: &str = "admin.rights.label.btn_show";
        pub const COL_CREATE: &str = "admin.rights.label.col_create";
        pub const COL_READ: &str = "admin.rights.label.col_read";
        pub const COL_UPDATE: &str = "admin.rights.label.col_update";
        pub const GO_TO_TABLE: &str = "admin.rights.label.go_to_table";
        pub const GROUP_LABEL: &str = "admin.rights.label.group_label";
        pub const GROUP_SHEET: &str = "admin.rights.label.group_sheet";
        pub const GROUP_SUBTITLE: &str = "admin.rights.label.group_subtitle";
        pub const RIGHT_CREATE: &str = "admin.rights.label.right_create";
        pub const RIGHT_DELETE: &str = "admin.rights.label.right_delete";
        pub const RIGHT_DELETE_OWN: &str = "admin.rights.label.right_delete_own";
        pub const RIGHT_READ: &str = "admin.rights.label.right_read";
        pub const RIGHT_READ_OWN: &str = "admin.rights.label.right_read_own";
        pub const RIGHT_UPDATE: &str = "admin.rights.label.right_update";
        pub const RIGHT_UPDATE_OWN: &str = "admin.rights.label.right_update_own";
        pub const SUBTITLE: &str = "admin.rights.label.subtitle";
        pub const TH_COLUMN: &str = "admin.rights.label.th_column";
        pub const TITLE: &str = "admin.rights.label.title";
        pub const TOC: &str = "admin.rights.label.toc";
    }
}
pub mod transfer {
    pub mod error {
        pub const CHANGED: &str = "admin.transfer.error.changed";
        pub const RANKED_MEMBERS: &str = "admin.transfer.error.ranked_members";
        pub const UNSUITABLE: &str = "admin.transfer.error.unsuitable";
    }
    pub mod info {
        pub const SUCCESS: &str = "admin.transfer.info.success";
    }
    pub mod label {
        pub const BTN_CANCEL: &str = "admin.transfer.label.btn_cancel";
        pub const BTN_CONFIRM: &str = "admin.transfer.label.btn_confirm";
        pub const FROM: &str = "admin.transfer.label.from";
        pub const GAINED_RIGHT: &str = "admin.transfer.label.gained_right";
        pub const LOSSES: &str = "admin.transfer.label.losses";
        pub const LOST_RIGHT: &str = "admin.transfer.label.lost_right";
        pub const NO_LOSS: &str = "admin.transfer.label.no_loss";
        pub const TITLE: &str = "admin.transfer.label.title";
        pub const TO: &str = "admin.transfer.label.to";
    }
}
pub mod user_created {
    pub mod info {
        pub const EMAIL_SENT: &str = "admin.user_created.info.email_sent";
    }
    pub mod label {
        pub const EMAIL_BODY: &str = "admin.user_created.label.email_body";
        pub const EMAIL_BTN: &str = "admin.user_created.label.email_btn";
        pub const EMAIL_GREETING: &str = "admin.user_created.label.email_greeting";
        pub const EMAIL_SUBJECT: &str = "admin.user_created.label.email_subject";
        pub const EMAIL_VALIDITY: &str = "admin.user_created.label.email_validity";
    }
}
