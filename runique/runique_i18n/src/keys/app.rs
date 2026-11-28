//! Keys of `app`.
pub mod error {
    pub const APP_ERROR: &str = "app.error.AppError";
    pub const BUILD: &str = "app.error.build";
    pub const CUSTOM: &str = "app.error.custom";
    pub const DATABASE: &str = "app.error.database";
    pub const FORBIDDEN: &str = "app.error.forbidden";
    pub const INTERNAL: &str = "app.error.internal";
    pub const INTERNAL_OCCURRED: &str = "app.error.internal_occurred";
    pub const INVALID_QUERY: &str = "app.error.invalid_query";
    pub const IO: &str = "app.error.io";
    pub const NOT_FOUND: &str = "app.error.not_found";
    pub const PATH_NOT_FOUND: &str = "app.error.path_not_found";
    pub const SOURCE: &str = "app.error.source";
    pub const TEMPLATE: &str = "app.error.template";
    pub const VALIDATION: &str = "app.error.validation";
}
pub mod label {
    pub const BAD_REQUEST: &str = "app.label.bad_request";
    pub const INTERNAL: &str = "app.label.internal";
    pub const NOT_FOUND: &str = "app.label.not_found";
    pub const TEMPLATE_IN: &str = "app.label.template_in";
}
pub mod log {
    pub const NOT_YES_NO_VALUE_USING_DEFAULT: &str = "app.log.not_yes_no_value_using_default";
    pub const RUNIQUE_WARNING_RUNIQUE_ENV_NEITHER_DEV: &str =
        "app.log.runique_warning_runique_env_neither_dev";
}
