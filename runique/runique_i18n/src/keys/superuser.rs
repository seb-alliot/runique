//! Keys of `superuser`.
pub mod error {
    pub const EMAIL_INVALID: &str = "superuser.error.email_invalid";
    pub const EMAIL_TAKEN: &str = "superuser.error.email_taken";
    pub const INVALID_PATH: &str = "superuser.error.invalid_path";
    pub const PASSWORD_MISMATCH: &str = "superuser.error.password_mismatch";
    pub const PASSWORD_TOO_SHORT: &str = "superuser.error.password_too_short";
    pub const PASSWORD_WEAK: &str = "superuser.error.password_weak";
    pub const PROVIDER_EXIT_CODE: &str = "superuser.error.provider_exit_code";
    pub const PROVIDER_LAUNCH_FAILED: &str = "superuser.error.provider_launch_failed";
    pub const PROVIDER_NOT_A_FILE: &str = "superuser.error.provider_not_a_file";
    pub const PROVIDER_NOT_FOUND: &str = "superuser.error.provider_not_found";
    pub const PROVIDER_OUTPUT_INVALID: &str = "superuser.error.provider_output_invalid";
    pub const PROVIDER_STDIN_FAILED: &str = "superuser.error.provider_stdin_failed";
    pub const PROVIDER_SYMLINK: &str = "superuser.error.provider_symlink";
    pub const PROVIDER_TIMEOUT: &str = "superuser.error.provider_timeout";
    pub const PROVIDER_WAIT_FAILED: &str = "superuser.error.provider_wait_failed";
    pub const USERNAME_EMPTY: &str = "superuser.error.username_empty";
    pub const USERNAME_TAKEN: &str = "superuser.error.username_taken";
}
pub mod info {
    pub const EMAIL_LINE: &str = "superuser.info.email_line";
    pub const ID_LINE: &str = "superuser.info.id_line";
    pub const REVIEW_ACTION: &str = "superuser.info.review_action";
    pub const REVIEW_ALGO: &str = "superuser.info.review_algo";
    pub const REVIEW_EMAIL: &str = "superuser.info.review_email";
    pub const REVIEW_PASSWORD: &str = "superuser.info.review_password";
    pub const REVIEW_USERNAME: &str = "superuser.info.review_username";
    pub const SUCCESS: &str = "superuser.info.success";
    pub const USERNAME_LINE: &str = "superuser.info.username_line";
}
pub mod label {
    pub const ALGO_ARGON2: &str = "superuser.label.algo_argon2";
    pub const ALGO_BCRYPT: &str = "superuser.label.algo_bcrypt";
    pub const ALGO_CUSTOM: &str = "superuser.label.algo_custom";
    pub const ALGO_PROMPT: &str = "superuser.label.algo_prompt";
    pub const ALGO_SCRYPT: &str = "superuser.label.algo_scrypt";
    pub const CHANGE_ALGO: &str = "superuser.label.change_algo";
    pub const CONFIRM_CREATE: &str = "superuser.label.confirm_create";
    pub const CONFIRM_PROMPT: &str = "superuser.label.confirm_prompt";
    pub const EMAIL_PROMPT: &str = "superuser.label.email_prompt";
    pub const MODIFY_PASSWORD: &str = "superuser.label.modify_password";
    pub const PASSWORD_PROMPT: &str = "superuser.label.password_prompt";
    pub const PROVIDER_PATH: &str = "superuser.label.provider_path";
    pub const TITLE: &str = "superuser.label.title";
    pub const USERNAME_PROMPT: &str = "superuser.label.username_prompt";
}
