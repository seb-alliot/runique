//! Keys of `build`.
pub mod error {
    pub const ACME_DOMAIN_MISSING: &str = "build.error.acme_domain_missing";
    pub const ACME_EMAIL_MISSING: &str = "build.error.acme_email_missing";
    pub const ADMIN_MODEL_INVALID: &str = "build.error.admin_model_invalid";
    pub const ADMIN_OVERRIDE_EXTENSION: &str = "build.error.admin_override_extension";
    pub const ADMIN_OVERRIDE_GLOBAL_PAGE: &str = "build.error.admin_override_global_page";
    pub const ADMIN_OVERRIDE_MISSING: &str = "build.error.admin_override_missing";
    pub const ADMIN_OVERRIDE_UNKNOWN_VARIABLE: &str = "build.error.admin_override_unknown_variable";
    pub const ADMIN_OVERRIDE_UNSAFE_CHAR: &str = "build.error.admin_override_unsafe_char";
    pub const ADMIN_PREFIX_EMPTY: &str = "build.error.admin_prefix_empty";
    pub const ADMIN_ROUTE_NAME_TAKEN: &str = "build.error.admin_route_name_taken";
    pub const ADMIN_ROUTES_WITHOUT_ADMIN: &str = "build.error.admin_routes_without_admin";
    pub const ADMIN_STYLESHEET_MISSING: &str = "build.error.admin_stylesheet_missing";
    pub const ADMIN_VIEW_REPLACED_TWICE: &str = "build.error.admin_view_replaced_twice";
    pub const CACHE_CONTROL_INVALID: &str = "build.error.cache_control_invalid";
    pub const CHECK_FAILED: &str = "build.error.check_failed";
    pub const COMPONENT_NOT_READY: &str = "build.error.component_not_ready";
    pub const CORS_WILDCARD_CREDENTIALS: &str = "build.error.cors_wildcard_credentials";
    pub const DB_CONNECT_FAILED: &str = "build.error.db_connect_failed";
    pub const DB_MISSING: &str = "build.error.db_missing";
    pub const DB_REQUIRED: &str = "build.error.db_required";
    pub const DRIVER_NOT_ENABLED: &str = "build.error.driver_not_enabled";
    pub const EXTERNAL_OUTPUTS_IGNORED: &str = "build.error.external_outputs_ignored";
    pub const LANG_UNKNOWN: &str = "build.error.lang_unknown";
    pub const LOCALES_INVALID: &str = "build.error.locales_invalid";
    pub const MAILER_MISSING: &str = "build.error.mailer_missing";
    pub const MEDIA_ROOT_UNUSABLE: &str = "build.error.media_root_unusable";
    pub const NOT_READY: &str = "build.error.not_ready";
    pub const PUBLIC_URL_MISSING: &str = "build.error.public_url_missing";
    pub const SCHEMA_NO_PRIMARY_KEY: &str = "build.error.schema_no_primary_key";
    pub const SECRET_KEY_WEAK: &str = "build.error.secret_key_weak";
    pub const SUBSCRIBER_ALREADY_INSTALLED: &str = "build.error.subscriber_already_installed";
    pub const TEMPLATE_FAILED: &str = "build.error.template_failed";
    pub const TIMEZONE_UNKNOWN: &str = "build.error.timezone_unknown";
    pub const VALIDATION_FAILED: &str = "build.error.validation_failed";
}
pub mod hint {
    pub const ACME_DOMAIN_MISSING: &str = "build.hint.acme_domain_missing";
    pub const ACME_EMAIL_MISSING: &str = "build.hint.acme_email_missing";
    pub const ADMIN_PREFIX_EMPTY: &str = "build.hint.admin_prefix_empty";
    pub const CACHE_CONTROL_INVALID: &str = "build.hint.cache_control_invalid";
    pub const DB_REQUIRED: &str = "build.hint.db_required";
    pub const MAILER_MISSING: &str = "build.hint.mailer_missing";
    pub const MEDIA_ROOT_UNUSABLE: &str = "build.hint.media_root_unusable";
    pub const PUBLIC_URL_MISSING: &str = "build.hint.public_url_missing";
    pub const SECRET_KEY_WEAK: &str = "build.hint.secret_key_weak";
}
pub mod label {
    pub const CONTEXT: &str = "build.label.context";
    pub const SUGGESTION: &str = "build.label.suggestion";
}
pub mod log {
    pub const DEBUG_IN_PROD: &str = "build.log.debug_in_prod";
    pub const HSTS_PRELOAD_SET_BUT_COMBO_INVALID: &str =
        "build.log.hsts_preload_set_but_combo_invalid";
    pub const LOCALES_MISSING: &str = "build.log.locales_missing";
    pub const LOCALES_PLACEHOLDERS: &str = "build.log.locales_placeholders";
    pub const LOCALES_UNKNOWN: &str = "build.log.locales_unknown";
    pub const ORPHAN_GROUP_RIGHTS_PRUNED_BOOT: &str = "build.log.orphan_group_rights_pruned_boot";
    pub const PRUNE_FAILED: &str = "build.log.prune_failed";
    pub const STATIC_FILES_ATTACHED: &str = "build.log.static_files_attached";
    pub const TEMPLATES_FAILED_LOAD: &str = "build.log.templates_failed_load";
    pub const TEMPLATES_LOADED: &str = "build.log.templates_loaded";
    pub const URL_REGISTRY: &str = "build.log.url_registry";
}
