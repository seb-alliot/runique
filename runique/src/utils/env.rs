use std::sync::LazyLock;

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

impl RuniqueEnv {
    fn from_env() -> Self {
        dotenvy::dotenv().ok();
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

/// Token de 4 chiffres calculé une seule fois au démarrage.
/// Utilisé comme cache-buster pour les assets statiques (`?v=XXXX`).
static CSS_TOKEN: LazyLock<String> = LazyLock::new(|| {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or_else(
            |_| "1000".to_string(),
            |d| (d.subsec_nanos() % 9000 + 1000).to_string(),
        )
});

pub fn css_token() -> String {
    CSS_TOKEN.clone()
}
