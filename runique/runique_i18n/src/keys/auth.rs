//! Keys of `auth`.
pub mod error {
    pub const LOGIN_CANNOT_SIGN_IN: &str = "auth.error.login_cannot_sign_in";
    pub const LOGIN_SESSION_FAILED: &str = "auth.error.login_session_failed";
}
pub mod log {
    pub const ACCOUNT_NOT_READ: &str = "auth.log.account_not_read";
    pub const ADMIN_ACCOUNT_NOT_READ: &str = "auth.log.admin_account_not_read";
    pub const ADMIN_GATE_LOOKUP_FAILED_ACTION_REFUSED: &str =
        "auth.log.admin_gate_lookup_failed_action_refused";
    pub const ADMIN_WRITE_REFUSED_DATABASE: &str = "auth.log.admin_write_refused_database";
    pub const DELETE_BLOCKED_FOREIGN_KEY: &str = "auth.log.delete_blocked_foreign_key";
    pub const FAILED_NO_RIGHTS_GRANTED_REQUEST: &str = "auth.log.failed_no_rights_granted_request";
    pub const GROUP_TRANSFER_REFUSED_RANKED_MEMBERS_ADMIN: &str =
        "auth.log.group_transfer_refused_ranked_members_admin";
    pub const GROUP_TRANSFER_REFUSED_UNSUITABLE_RECEIVER: &str =
        "auth.log.group_transfer_refused_unsuitable_receiver";
    pub const HASH_FAILED: &str = "auth.log.hash_failed";
    pub const LOADING_ACCOUNT_RIGHTS_FAILED_NO_RIGHTS: &str =
        "auth.log.loading_account_rights_failed_no_rights";
    pub const PASSWORD_INIT_CALLED_MULTIPLE_TIMES_INITIAL: &str =
        "auth.log.password_init_called_multiple_times_initial";
    pub const PASSWORD_REHASH_FAILED: &str = "auth.log.password_rehash_failed";
    pub const REFUSED_ACCOUNT_STILL_ANSWERS_GROUPS: &str =
        "auth.log.refused_account_still_answers_groups";
    pub const SAVING_REHASHED_PASSWORD_FAILED: &str = "auth.log.saving_rehashed_password_failed";
}
