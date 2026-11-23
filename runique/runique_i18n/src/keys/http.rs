//! Keys of `http`.
pub mod error {
    pub const CODE_400: &str = "http.error.400";
    pub const CODE_404: &str = "http.error.404";
    pub const CODE_413: &str = "http.error.413";
    pub const CODE_429: &str = "http.error.429";
    pub const CODE_500: &str = "http.error.500";
    pub const CODE_503: &str = "http.error.503";
    pub const CRITICAL_CONTACT: &str = "http.error.critical_contact";
    pub const CRITICAL_TEXT: &str = "http.error.critical_text";
}
pub mod label {
    pub const CODE_400: &str = "http.label.400";
    pub const CODE_404: &str = "http.label.404";
    pub const CODE_413: &str = "http.label.413";
    pub const CODE_429: &str = "http.label.429";
    pub const CODE_500: &str = "http.label.500";
    pub const CODE_503: &str = "http.label.503";
    pub const BACK_HOME: &str = "http.label.back_home";
    pub const CRITICAL: &str = "http.label.critical";
}
