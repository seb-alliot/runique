//! Keys of `middleware`.
pub mod error {
    pub const BAD_REQUEST: &str = "middleware.error.bad_request";
    pub const FORBIDDEN_REDIRECT: &str = "middleware.error.forbidden_redirect";
    pub const SLOT_COLLISION: &str = "middleware.error.slot_collision";
}
pub mod log {
    pub const HONEYPOT_FIELD_NAME_INSERT_INTO_SESSION: &str =
        "middleware.log.honeypot_field_name_insert_into_session";
    pub const HOST_REJECTED_NO_HOST_HEADER_URI: &str =
        "middleware.log.host_rejected_no_host_header_uri";
    pub const HOST_REJECTED_NOT_ALLOWLIST: &str = "middleware.log.host_rejected_not_allowlist";
    pub const MIDDLEWARE_SLOT: &str = "middleware.log.middleware_slot";
    pub const MIDDLEWARE_STACK: &str = "middleware.log.middleware_stack";
    pub const OPEN_REDIRECT_BLOCKED: &str = "middleware.log.open_redirect_blocked";
    pub const RATE_LIMITED: &str = "middleware.log.rate_limited";
}
