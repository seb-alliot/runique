//! Environnement d'exécution — mode debug/production, chargement `.env`, token CSS.
use std::{path::Path, sync::LazyLock};

/// Mode d'exécution de l'application.
///
/// Déterminé une seule fois au démarrage depuis `DEBUG` dans `.env`.
/// - `DEBUG=true` ou `DEBUG=1` → [`Development`](RuniqueEnv::Development)
/// - Toute autre valeur ou absent → [`Production`](RuniqueEnv::Production)
///
/// Utiliser [`is_debug()`] pour accéder au mode depuis n'importe où.
pub enum RuniqueEnv {
    Development,
    Production,
}

pub fn load_env(files: Vec<&str>) {
    files.iter().for_each(|file| {
        if Path::new(file).exists() {
            if let Err(e) = dotenvy::from_path_override(file) {
                eprintln!(
                    "Impossible de charger {} : {}, config par default via .env",
                    file, e
                );
            }
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

/// Retourne `true` si l'application tourne en mode développement (`DEBUG=true`).
///
/// Lu une seule fois au démarrage depuis `.env`, stocké en `LazyLock`.
/// Disponible partout dans le framework sans passer de paramètre.
///
/// # Exemple
/// ```rust,ignore
/// use runique::prelude::*;
///
/// if is_debug() {
///     println!("Mode développement actif");
/// }
/// ```
#[must_use]
pub fn is_debug() -> bool {
    matches!(*ENV, RuniqueEnv::Development)
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
