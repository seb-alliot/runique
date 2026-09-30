//! Execution environment — debug mode, CSS token.
use std::sync::LazyLock;

/// `DEBUG` from `.env`, read once at startup.
static DEBUG: LazyLock<bool> = LazyLock::new(|| debug_from(std::env::var("DEBUG").ok().as_deref()));

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

/// The yes/no environment variable `key` (see [`flag_from`]), or `default` when
/// it isn't set. A value that's neither yes nor no gets a warning and the
/// default, rather than being silently taken for one or the other.
#[must_use]
pub fn env_flag(key: &str, default: bool) -> bool {
    let Ok(raw) = std::env::var(key) else {
        return default;
    };
    flag_from(&raw).unwrap_or_else(|| {
        tracing::warn!(key, value = %raw, default, "not a yes/no value, using the default");
        default
    })
}

/// A keyword-valued environment variable (`DB_ENGINE=Postgres`,
/// `EMAIL_BACKEND=Console`), trimmed and lowercased so it compares the same
/// way everywhere. Never for secrets, URLs, paths or table names: those are
/// case-sensitive.
#[must_use]
pub fn env_keyword(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
}

/// Whether a `DEBUG` value turns debug mode on (see [`flag_from`]); unset or
/// unreadable means off.
#[must_use]
pub fn debug_from(value: Option<&str>) -> bool {
    value.and_then(flag_from).unwrap_or(false)
}

/// Returns `true` when `DEBUG` turns debug mode on (see [`debug_from`]).
///
/// Read once at startup, stored in a `LazyLock`, available everywhere in the
/// framework without passing parameters.
///
/// # Example
/// ```rust,ignore
/// use runique::prelude::*;
///
/// if is_debug() {
///     println!("Development mode active");
/// }
/// ```
#[must_use]
pub fn is_debug() -> bool {
    *DEBUG
}

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

static CSS_TOKEN: LazyLock<String> = LazyLock::new(|| {
    let static_dir = std::env::var("STATICFILES_DIRS").unwrap_or_else(|_| "static".to_string());
    hash_static_files(&static_dir).unwrap_or_else(|| "1000".to_string())
});

fn hash_static_files(dir: &str) -> Option<String> {
    let mut hasher = DefaultHasher::new();
    let mut found = false;

    for entry in walkdir::WalkDir::new(dir).sort_by_file_name() {
        let entry = entry.ok()?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "css" || e == "js") {
            std::fs::read_to_string(path).ok()?.hash(&mut hasher);
            found = true;
        }
    }

    if found {
        Some(format!("{:08x}", hasher.finish()))
    } else {
        None
    }
}

/// Returns the cache-busting token appended as `?v=` to static asset URLs.
///
/// Computed once (`LazyLock`) as a hash of every `.css`/`.js` file's content under
/// `STATICFILES_DIRS` (default `"static"`), so it changes whenever those files
/// change. Falls back to `"1000"` if the directory has no matching files.
pub fn css_token() -> String {
    CSS_TOKEN.clone()
}

#[cfg(test)]
mod debug_tests {
    use super::{debug_from, flag_from};

    #[test]
    fn flags_are_read_whatever_the_case() {
        for on in ["true", "True", "TRUE", "1", "yes", "On", " true "] {
            assert_eq!(flag_from(on), Some(true), "{on:?}");
        }
        for off in ["false", "False", "0", "no", "OFF"] {
            assert_eq!(flag_from(off), Some(false), "{off:?}");
        }
        for neither in ["", "treu", "prod", "2"] {
            assert_eq!(flag_from(neither), None, "{neither:?}");
        }
    }

    #[test]
    fn debug_values_are_read_whatever_the_case() {
        for on in ["true", "True", "TRUE", "1", "yes", "On", " true "] {
            assert!(debug_from(Some(on)), "{on:?} should turn debug on");
        }
        for off in ["false", "False", "0", "no", "", "prod"] {
            assert!(!debug_from(Some(off)), "{off:?} should leave debug off");
        }
        assert!(!debug_from(None));
    }
}
