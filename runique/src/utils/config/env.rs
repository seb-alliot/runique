//! Execution environment — debug/production mode, `.env` loading, CSS token.
use std::{path::Path, sync::LazyLock};

/// Application execution mode.
///
/// Determined once at startup from `DEBUG` in `.env`.
/// - `DEBUG=true` or `DEBUG=1` → [`Development`](RuniqueEnv::Development)
/// - Any other value or absent → [`Production`](RuniqueEnv::Production)
///
/// Use [`is_debug()`] to access the mode from anywhere.
pub enum RuniqueEnv {
    Development,
    Production,
}

pub fn load_env(files: Vec<&str>) {
    files.iter().for_each(|file| {
        if Path::new(file).exists()
            && let Some(level) = crate::utils::runique_log::get_log().password_init
        {
            crate::runique_log!(
                level,
                "password_init() called multiple times — initial configuration is kept"
            );
        }
    });
}

impl RuniqueEnv {
    fn from_env() -> Self {
        match std::env::var("DEBUG").as_deref() {
            Ok("true" | "1") => Self::Development,
            _ => Self::Production,
        }
    }
}

static ENV: LazyLock<RuniqueEnv> = LazyLock::new(RuniqueEnv::from_env);

/// Returns `true` if the application is running in development mode (`DEBUG=true`).
///
/// Read once at startup from `.env`, stored in `LazyLock`.
/// Available everywhere in the framework without passing parameters.
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
    matches!(*ENV, RuniqueEnv::Development)
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

pub fn css_token() -> String {
    CSS_TOKEN.clone()
}
