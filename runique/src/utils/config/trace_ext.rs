//! `TraceResult` — extension trait to log a swallowed `Result` with the caller's
//! `file:line`, then discard the error.
//!
//! Replaces silent `.ok()` / `.unwrap_or_default()` on fallible operations:
//!
//! ```rust,ignore
//! // before — silent
//! renderer.render_field(&hp).unwrap_or_default()
//! // after — logs file:line + error if the category is active, else zero cost
//! renderer.render_field(&hp)
//!     .trace_or_default(get_log().forms.as_ref().and_then(|f| f.render), "honeypot render")
//! ```
use tracing::Level;

pub trait TraceResult<T> {
    /// Logs the error (caller `file:line` + `ctx`) at `level` if `Some`, returns `Option<T>`.
    #[track_caller]
    fn trace(self, level: Option<Level>, ctx: &'static str) -> Option<T>;

    /// Same, but returns `T::default()` instead of `None`.
    #[track_caller]
    fn trace_or_default(self, level: Option<Level>, ctx: &'static str) -> T
    where
        T: Default;
}

impl<T, E: std::fmt::Debug> TraceResult<T> for Result<T, E> {
    #[track_caller]
    fn trace(self, level: Option<Level>, ctx: &'static str) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                if let Some(level) = level {
                    let loc = std::panic::Location::caller();
                    crate::runique_log!(level, error = ?e, at = %loc, "{}", ctx);
                }
                None
            }
        }
    }

    #[track_caller]
    fn trace_or_default(self, level: Option<Level>, ctx: &'static str) -> T
    where
        T: Default,
    {
        match self {
            Ok(v) => v,
            Err(e) => {
                if let Some(level) = level {
                    let loc = std::panic::Location::caller();
                    crate::runique_log!(level, error = ?e, at = %loc, "{}", ctx);
                }
                T::default()
            }
        }
    }
}
