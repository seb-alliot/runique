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

    /// Like [`trace`], but **never silent**: when the category is disabled (`None`),
    /// the error is still logged at `default`. Use only on sites where a silent
    /// failure breaks a security guarantee (session fixation, exclusive login) or
    /// leaves an operational black hole (login not persisted, reset email lost).
    ///
    /// An explicit category level still wins, so the user can raise/lower it globally.
    ///
    /// [`trace`]: TraceResult::trace
    #[track_caller]
    fn trace_or(self, level: Option<Level>, default: Level, ctx: &'static str) -> Option<T>;

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
    fn trace_or(self, level: Option<Level>, default: Level, ctx: &'static str) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                let loc = std::panic::Location::caller();
                crate::runique_log!(level.unwrap_or(default), error = ?e, at = %loc, "{}", ctx);
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

#[cfg(test)]
mod tests {
    use super::TraceResult;
    use tracing::Level;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn trace_logs_error_and_caller_location_then_returns_none() {
        let r: Result<i32, &str> = Err("kaboom");
        assert!(r.trace(Some(Level::WARN), "while doing X").is_none());
        assert!(logs_contain("while doing X"));
        assert!(logs_contain("kaboom"));
        // #[track_caller] → the logged `at` points back to this test file.
        assert!(logs_contain("trace_ext.rs"));
    }

    #[traced_test]
    #[test]
    fn trace_is_silent_when_level_is_none() {
        let r: Result<i32, &str> = Err("hidden error");
        assert!(r.trace(None, "should not appear").is_none());
        assert!(!logs_contain("should not appear"));
        assert!(!logs_contain("hidden error"));
    }

    #[traced_test]
    #[test]
    fn trace_passes_ok_through_without_logging() {
        let r: Result<i32, &str> = Ok(7);
        assert_eq!(r.trace(Some(Level::WARN), "ok ctx"), Some(7));
        assert!(!logs_contain("ok ctx"));
    }

    #[traced_test]
    #[test]
    fn trace_or_is_never_silent_even_when_category_disabled() {
        let r: Result<(), &str> = Err("floor me");
        // category off (None) but the floor still emits at WARN
        assert!(r.trace_or(None, Level::WARN, "floored ctx").is_none());
        assert!(logs_contain("floored ctx"));
        assert!(logs_contain("floor me"));
    }

    #[traced_test]
    #[test]
    fn trace_or_default_returns_default_and_logs() {
        let r: Result<u32, &str> = Err("bad value");
        assert_eq!(r.trace_or_default(Some(Level::INFO), "to default"), 0);
        assert!(logs_contain("to default"));
        assert!(logs_contain("bad value"));
    }

    #[traced_test]
    #[test]
    fn runique_log_macro_emits_at_the_given_level() {
        crate::runique_log!(Level::INFO, item = 3, "macro speaks");
        assert!(logs_contain("macro speaks"));
    }

    #[traced_test]
    #[serial_test::serial]
    #[test]
    fn gated_site_emits_only_when_its_category_is_configured() {
        use crate::utils::runique_log::{RuniqueLog, get_log, log_init, reset_log_for_test};

        // Disabled category → the canonical `if let Some(level) = …` gate stays silent.
        reset_log_for_test();
        log_init(RuniqueLog::new());
        if let Some(level) = get_log().db.as_ref().and_then(|d| d.query) {
            crate::runique_log!(level, "db query while disabled");
        }
        assert!(!logs_contain("db query while disabled"));

        // Enabled category → the same gate emits.
        reset_log_for_test();
        log_init(RuniqueLog::new().db(|d| d.query(Level::WARN)));
        if let Some(level) = get_log().db.as_ref().and_then(|d| d.query) {
            crate::runique_log!(level, "db query while enabled");
        }
        assert!(logs_contain("db query while enabled"));
        reset_log_for_test();
    }
}
