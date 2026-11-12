//! Execution environment — `RUNIQUE_ENV`, `DEBUG`, CSS token.
use std::sync::LazyLock;

/// A variable from the environment or `.env`, for the two flags below: a
/// `LazyLock` keeps its first answer, and `main` may ask before
/// `RuniqueConfig::from_env()` has loaded the file.
fn read_after_dotenv(key: &str) -> Option<String> {
    dotenvy::dotenv().ok();
    std::env::var(key).ok()
}

static DEBUG: LazyLock<bool> = LazyLock::new(|| debug_from(read_after_dotenv("DEBUG").as_deref()));

static DEV: LazyLock<bool> =
    LazyLock::new(|| dev_from(read_after_dotenv("RUNIQUE_ENV").as_deref()));

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

/// Whether a `RUNIQUE_ENV` value selects development: `dev`, whatever its case
/// and surrounding spaces. Unset or anything else means production.
#[must_use]
pub fn dev_from(value: Option<&str>) -> bool {
    value.is_some_and(|v| v.trim().eq_ignore_ascii_case("dev"))
}

/// Whether a `RUNIQUE_ENV` value is one of `dev` and `prod`.
#[must_use]
pub fn runique_env_is_known(value: &str) -> bool {
    let value = value.trim();
    value.eq_ignore_ascii_case("dev") || value.eq_ignore_ascii_case("prod")
}

/// Returns `true` when `DEBUG` turns on the detailed error pages and every log
/// channel (see [`debug_from`]). It changes nothing else: use [`is_dev`] for
/// the behaviors that differ between development and production.
///
/// Read once, stored in a `LazyLock`, available everywhere in the framework
/// without passing parameters.
#[must_use]
pub fn is_debug() -> bool {
    *DEBUG
}

/// Returns `true` when `RUNIQUE_ENV=dev` (see [`dev_from`]).
///
/// Read once, stored in a `LazyLock`, available everywhere in the framework
/// without passing parameters.
///
/// # Example
/// ```rust,ignore
/// use runique::prelude::*;
///
/// if is_dev() {
///     println!("Development mode active");
/// }
/// ```
#[must_use]
pub fn is_dev() -> bool {
    *DEV
}

use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

static CSS_TOKEN: LazyLock<String> = LazyLock::new(|| {
    let static_dir = std::env::var("STATICFILES_DIRS").unwrap_or_else(|_| "static".to_string());
    let runique_dir = crate::config::StaticConfig::from_env().static_runique_path;
    asset_token(&[&static_dir, &runique_dir])
});

/// The token of several static folders. Runique's folder counts as much as the
/// project's: its files are served `immutable` under the same `?v=`, so a
/// token blind to them leaves browsers and CDNs on the old file after an
/// upgrade, which its new SRI hash then blocks.
fn asset_token(dirs: &[&str]) -> String {
    let hashes: String = dirs.iter().filter_map(|d| hash_static_files(d)).collect();
    if hashes.is_empty() {
        "1000".to_string()
    } else {
        hashes
    }
}

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
/// `STATICFILES_DIRS` (default `"static"`) and Runique's own static folder, so it
/// changes whenever those files change. Falls back to `"1000"` if neither has
/// matching files.
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
    fn only_dev_selects_development() {
        for on in ["dev", "DEV", "Dev", " dev ", "\tdev\n"] {
            assert!(super::dev_from(Some(on)), "{on:?} should select dev");
        }
        for off in [
            "prod",
            "PROD",
            "development",
            "dev2",
            "de v",
            "true",
            "1",
            "",
            "  ",
        ] {
            assert!(!super::dev_from(Some(off)), "{off:?} should stay prod");
        }
        assert!(!super::dev_from(None));
    }

    #[test]
    fn only_dev_and_prod_are_known_environments() {
        for known in ["dev", "DEV", " dev ", "prod", "PROD", "Prod", " prod\t"] {
            assert!(super::runique_env_is_known(known), "{known:?}");
        }
        for unknown in [
            "",
            " ",
            "development",
            "production",
            "staging",
            "true",
            "dev prod",
            "devs",
        ] {
            assert!(!super::runique_env_is_known(unknown), "{unknown:?}");
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

    /// The `?v=` token hashes every `.css` and `.js` under the static folder,
    /// in name order — and nothing else. Written from cargo-mutants survivors
    /// (2026-10-08): env.rs:82, 88, 107 (the asset version behind the SRI bug).
    #[test]
    fn the_asset_token_hashes_css_and_js_only() {
        use std::hash::{DefaultHasher, Hash, Hasher};
        let dir = std::env::temp_dir().join(format!("rq_token_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("css")).unwrap();
        std::fs::write(dir.join("css/a.css"), "body{}").unwrap();
        std::fs::write(dir.join("app.js"), "go()").unwrap();
        std::fs::write(dir.join("logo.svg"), "<svg/>").unwrap();

        let mut expected = DefaultHasher::new();
        // walkdir, sorted by name: app.js, css/a.css (logo.svg is skipped).
        "go()".to_string().hash(&mut expected);
        "body{}".to_string().hash(&mut expected);
        let token = super::hash_static_files(dir.to_str().unwrap());

        std::fs::write(dir.join("logo.svg"), "<svg changed/>").unwrap();
        let same = super::hash_static_files(dir.to_str().unwrap());
        let empty = std::env::temp_dir().join(format!("rq_token_none_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&empty).unwrap();
        std::fs::write(empty.join("logo.svg"), "<svg/>").unwrap();
        let none = super::hash_static_files(empty.to_str().unwrap());
        let _ = (
            std::fs::remove_dir_all(&dir),
            std::fs::remove_dir_all(&empty),
        );

        assert_eq!(token, Some(format!("{:08x}", expected.finish())));
        assert_eq!(same, token, "an image change doesn't move the token");
        assert_eq!(none, None, "no stylesheet or script: no token");
    }

    #[test]
    fn the_asset_token_follows_every_folder() {
        let root = std::env::temp_dir().join(format!("rq_token_{}", uuid::Uuid::new_v4()));
        let (project, framework) = (root.join("project"), root.join("framework"));
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&framework).unwrap();
        std::fs::write(project.join("site.css"), "a{}").unwrap();
        std::fs::write(framework.join("admin.css"), "b{}").unwrap();
        let (p, f) = (project.to_str().unwrap(), framework.to_str().unwrap());

        let before = super::asset_token(&[p, f]);
        std::fs::write(framework.join("admin.css"), "b{color:red}").unwrap();
        let framework_changed = super::asset_token(&[p, f]);
        std::fs::write(project.join("site.css"), "a{color:red}").unwrap();
        let project_changed = super::asset_token(&[p, f]);
        let one_missing = super::asset_token(&[p, "/rq_no_such_folder"]);
        let none = super::asset_token(&["/rq_no_such_folder"]);
        let _ = std::fs::remove_dir_all(&root);

        assert_ne!(framework_changed, before, "Runique's own file moves it");
        assert_ne!(project_changed, framework_changed, "the project's too");
        assert!(!one_missing.is_empty(), "a folder missing leaves the other");
        assert_eq!(none, "1000", "no folder: the fallback");
    }

    #[test]
    fn css_token_is_the_cached_hash() {
        let token = super::css_token();
        assert!(
            !token.is_empty() && token.chars().all(|c| c.is_ascii_hexdigit()),
            "{token}"
        );
        assert_eq!(super::css_token(), token, "computed once");
    }
}
