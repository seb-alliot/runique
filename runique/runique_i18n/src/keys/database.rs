//! Keys of `database`.
pub mod error {
    pub const KEYS_WITHOUT_ENGINE: &str = "database.error.keys_without_engine";
    pub const LIST_POSTGRES_ONLY: &str = "database.error.list_postgres_only";
    pub const LIST_VALUE_NOT_IN_ENUM: &str = "database.error.list_value_not_in_enum";
    pub const REASON_NO_DATABASE: &str = "database.error.reason_no_database";
    pub const REASON_SQLITE_NO_FILE: &str = "database.error.reason_sqlite_no_file";
    pub const REASON_UNKNOWN_START: &str = "database.error.reason_unknown_start";
    pub const REASON_UNREADABLE: &str = "database.error.reason_unreadable";
    pub const SEARCH_ONE_MULTIPLE: &str = "database.error.search_one_multiple";
    pub const UNSUPPORTED_URL_SCHEME: &str = "database.error.unsupported_url_scheme";
    pub const URL_FAILED_KEYS_UNUSABLE: &str = "database.error.url_failed_keys_unusable";
    pub const URL_FAILED_TRYING_KEYS: &str = "database.error.url_failed_trying_keys";
    pub const URL_INCOMPLETE_KEYS_UNUSABLE: &str = "database.error.url_incomplete_keys_unusable";
    pub const URL_INCOMPLETE_NO_KEYS: &str = "database.error.url_incomplete_no_keys";
    pub const URL_INCOMPLETE_USING_KEYS: &str = "database.error.url_incomplete_using_keys";
    pub const URL_SQLITE_DEFAULT_FILE: &str = "database.error.url_sqlite_default_file";
}
pub mod info {
    pub const URL_INCOMPLETE_REMINDER: &str = "database.info.url_incomplete_reminder";
}
pub mod log {
    pub const CONNECTING_DATABASE: &str = "database.log.connecting_database";
    pub const DATABASE_CONNECTED_SUCCESSFULLY: &str =
        "database.log.database_connected_successfully";
    pub const DATABASE_CONNECTION_FAILED: &str = "database.log.database_connection_failed";
    pub const ENGINE: &str = "database.log.engine";
    pub const READ_RESET_TOKEN: &str = "database.log.read_reset_token";
    pub const URL: &str = "database.log.url";
}
