//! What the commands take from `runique::utils`, copied so that the command
//! builds without the framework.
pub mod password;

pub(crate) mod aliases {
    /// The connection the commands work on.
    pub(crate) type ADb = sea_orm::DatabaseConnection;
}

pub(crate) mod config {
    pub(crate) mod env {
        /// Reads a yes/no value, whatever its case and surrounding spaces: `true`,
        /// `1`, `yes`, `on` or `false`, `0`, `no`, `off`. `None` for anything else.
        ///
        /// Rust's own `str::parse::<bool>()` only takes `true`/`false` exactly: with it,
        /// `ENFORCE_HTTPS=True` was quietly read as the default, and HTTPS wasn't enforced.
        #[must_use]
        pub fn flag_from(value: &str) -> Option<bool> {
            match value.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" | "on" => Some(true),
                "false" | "0" | "no" | "off" => Some(false),
                _ => None,
            }
        }
    }
}
