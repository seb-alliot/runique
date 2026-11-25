//! Keys of `build`.
pub mod error {
    pub const ACME_DOMAIN_MISSING: &str = "build.error.acme_domain_missing";
    pub const ACME_EMAIL_MISSING: &str = "build.error.acme_email_missing";
    pub const ADMIN_PREFIX_EMPTY: &str = "build.error.admin_prefix_empty";
    pub const CACHE_CONTROL_INVALID: &str = "build.error.cache_control_invalid";
    pub const CHECK_FAILED: &str = "build.error.check_failed";
    pub const COMPONENT_NOT_READY: &str = "build.error.component_not_ready";
    pub const DB_MISSING: &str = "build.error.db_missing";
    pub const DB_REQUIRED: &str = "build.error.db_required";
    pub const DRIVER_NOT_ENABLED: &str = "build.error.driver_not_enabled";
    pub const EXTERNAL_OUTPUTS_IGNORED: &str = "build.error.external_outputs_ignored";
    pub const LANG_UNKNOWN: &str = "build.error.lang_unknown";
    pub const LOCALES_INVALID: &str = "build.error.locales_invalid";
    pub const MAILER_MISSING: &str = "build.error.mailer_missing";
    pub const MEDIA_ROOT_UNUSABLE: &str = "build.error.media_root_unusable";
    pub const PUBLIC_URL_MISSING: &str = "build.error.public_url_missing";
    pub const SECRET_KEY_WEAK: &str = "build.error.secret_key_weak";
    pub const SUBSCRIBER_ALREADY_INSTALLED: &str = "build.error.subscriber_already_installed";
    pub const TEMPLATE_FAILED: &str = "build.error.template_failed";
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
    pub const LOCALES_MISSING: &str = "build.log.locales_missing";
    pub const LOCALES_PLACEHOLDERS: &str = "build.log.locales_placeholders";
    pub const LOCALES_UNKNOWN: &str = "build.log.locales_unknown";
    pub const PRUNE_FAILED: &str = "build.log.prune_failed";
}
